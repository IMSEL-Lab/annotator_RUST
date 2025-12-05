slint::include_modules!();

mod config;
mod classes;
mod export;
mod auto_resize;
mod hierarchy;
mod state;
mod utils;
mod callbacks;

use state::{
    DatasetEntry, DatasetFile, DatasetFileEntry, DatasetState, DrawState, ResizeState,
    StoredAnnotation, UndoHistory, ViewState,
    // Functions
    ann_to_stored, apply_view_state, create_dataset_from_folder, generate_path_commands,
    get_view_state, label_path_for, load_dataset, load_image_from_entry, load_yolo_annotations,
    next_id_from_annotations, parse_vertices, replace_annotations, save_all, save_current_state,
    sizes_close, snapshot_annotations, state_path_for,
};
use utils::{parse_color, placeholder_image};

use slint::Model;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // Load configuration
    let config = Rc::new(RefCell::new(config::load_config()));

    // Load class definitions
    // Always prefer the bundled default classes.yaml in the repo root; users can
    // still override by replacing that file. This avoids stale paths in the
    // persisted config pointing elsewhere.
    let classes = Rc::new(RefCell::new(classes::load_classes(None)));

    // Apply initial theme from config
    let _theme_name = config.borrow().appearance.theme.clone();
    // Theme will be set via callback later if needed
    // For now, it defaults to dark theme in the Slint code

    // Populate class items for the sidebar (flat mode)
    let class_items: Vec<ClassItem> = classes
        .borrow()
        .classes
        .iter()
        .map(|c| ClassItem {
            id: c.id,
            name: c.name.clone().into(),
            color: c
                .color
                .as_ref()
                .and_then(|hex| parse_color(hex))
                .unwrap_or(slint::Color::from_rgb_u8(128, 128, 128))
                .into(),
            shortcut: c.shortcut.clone().unwrap_or_default().into(),
        })
        .collect();
    ui.set_class_items(slint::ModelRc::new(slint::VecModel::from(class_items)));

    // Initialize hierarchy navigation if hierarchy exists
    let hierarchy_navigator = Rc::new(RefCell::new(
        hierarchy::HierarchyNavigator::new(&classes.borrow())
    ));

    let is_hierarchical = hierarchy_navigator.borrow().is_hierarchical();
    ui.set_hierarchy_mode(is_hierarchical);

    if is_hierarchical {
        println!("✓ Hierarchical class selection enabled ({} levels)",
                 hierarchy_navigator.borrow().max_depth());

        // Set initial hierarchy options
        let navigator = hierarchy_navigator.borrow();
        let options: Vec<HierarchyOption> = navigator.get_current_level_nodes()
            .iter()
            .map(|node| HierarchyOption {
                key: node.key as i32,
                label: node.label.clone().into(),
                is_leaf: node.id.is_some(),
            })
            .collect();
        ui.set_hierarchy_options(slint::ModelRc::new(slint::VecModel::from(options)));
        ui.set_hierarchy_prompt(navigator.get_prompt().into());
        ui.set_hierarchy_breadcrumb("".into());
    }

    // Debug: Terminal commands for adjusting sidebar
    {
        let ui_handle = ui.as_weak();
        std::thread::spawn(move || {
            use std::io::{self, BufRead};
            let stdin = io::stdin();
            println!("\n=== SIDEBAR DEBUG COMMANDS ===");
            println!("Type commands to adjust sidebar:");
            println!("  width <number>  - Set sidebar width in pixels (e.g., 'width 300')");
            println!("  hide            - Hide sidebar");
            println!("  show            - Show sidebar");
            println!("  help            - Show this help");
            println!("==============================\n");

            for line in stdin.lock().lines() {
                if let Ok(line) = line {
                    let parts: Vec<&str> = line.trim().split_whitespace().collect();
                    if parts.is_empty() {
                        continue;
                    }

                    if let Some(ui) = ui_handle.upgrade() {
                        match parts[0] {
                            "width" | "w" => {
                                if let Some(width_str) = parts.get(1) {
                                    if let Ok(width) = width_str.parse::<f32>() {
                                        ui.set_sidebar_width(width);
                                        println!("✓ Sidebar width set to {}px", width);
                                    } else {
                                        println!("✗ Invalid number. Usage: width <number>");
                                    }
                                } else {
                                    println!("✗ Usage: width <number>");
                                }
                            }
                            "hide" | "h" => {
                                ui.set_sidebar_visible(false);
                                println!("✓ Sidebar hidden");
                            }
                            "show" | "s" => {
                                ui.set_sidebar_visible(true);
                                println!("✓ Sidebar shown");
                            }
                            "help" | "?" => {
                                println!("\n=== SIDEBAR DEBUG COMMANDS ===");
                                println!("  width <number>  - Set sidebar width in pixels");
                                println!("  hide            - Hide sidebar");
                                println!("  show            - Show sidebar");
                                println!("  help            - Show this help");
                                println!("==============================\n");
                            }
                            _ => {
                                println!("✗ Unknown command '{}'. Type 'help' for commands.", parts[0]);
                            }
                        }
                    } else {
                        break; // UI closed
                    }
                }
            }
        });
    }

    // Add callback for getting class name
    {
        let classes_ref = classes.clone();
        ui.on_get_class_name(move |class_id| {
            classes::get_class_name(&classes_ref.borrow(), class_id).into()
        });
    }

    // Initialize settings from config
    ui.set_theme_setting(config.borrow().appearance.theme.clone().into());
    ui.set_enable_points_setting(config.borrow().annotation_modes.enable_points);
    ui.set_enable_bboxes_setting(config.borrow().annotation_modes.enable_bboxes);
    ui.set_enable_polygons_setting(config.borrow().annotation_modes.enable_polygons);
    ui.set_randomize_dataset_setting(config.borrow().dataset.randomize_order);

    // Add callback for applying settings
    {
        let config_ref = config.clone();
        ui.on_apply_settings(move |theme, enable_points, enable_bboxes, enable_polygons, randomize| {
            let mut cfg = config_ref.borrow_mut();
            cfg.appearance.theme = theme.to_string();
            cfg.annotation_modes.enable_points = enable_points;
            cfg.annotation_modes.enable_bboxes = enable_bboxes;
            cfg.annotation_modes.enable_polygons = enable_polygons;
            cfg.dataset.randomize_order = randomize;

            // Save to disk
            if let Err(e) = config::save_config(&cfg) {
                eprintln!("Failed to save config: {}", e);
            }
        });
    }

    let draw_state = Rc::new(RefCell::new(DrawState::new()));
    let resize_state = Rc::new(RefCell::new(ResizeState::new()));
    let undo_history = Rc::new(RefCell::new(UndoHistory::new(50))); // Max 50 undo steps
    let clipboard: Rc<RefCell<Vec<Annotation>>> = Rc::new(RefCell::new(Vec::new())); // Annotation clipboard for copy/paste (supports multiple)
    let annotations = std::rc::Rc::new(slint::VecModel::from(Vec::<Annotation>::new()));
    ui.set_annotations(annotations.clone().into());

    // Add callback for hierarchy navigation
    {
        let navigator_ref = hierarchy_navigator.clone();
        let ui_handle = ui.as_weak();
        let annotations_ref = annotations.clone();

        ui.on_hierarchy_navigate(move |key| {
            let mut navigator = navigator_ref.borrow_mut();
            let ui = match ui_handle.upgrade() {
                Some(ui) => ui,
                None => return,
            };

            if key == 0 {
                // Navigate up (ESC key)
                navigator.navigate_up();
            } else if (1..=5).contains(&key) {
                // Navigate down (1-5 keys)
                if let Some(class_id) = navigator.navigate_down(key as u8) {
                    // Reached a leaf node - assign class
                    ui.set_current_class(class_id);

                    // Classify any selected annotations
                    let count = annotations_ref.row_count();
                    let mut changed = false;
                    for i in 0..count {
                        if let Some(mut ann) = annotations_ref.row_data(i) {
                            if ann.selected {
                                ann.class = class_id;
                                annotations_ref.set_row_data(i, ann);
                                changed = true;
                            }
                        }
                    }

                    if changed {
                        ui.set_status_text(format!("Assigned class {} to selected annotations", class_id).into());
                    } else {
                        ui.set_status_text(format!("Class {} selected", class_id).into());
                    }

                    // Navigator has auto-reset, so return to root
                }
            }

            // Update UI with current hierarchy state
            let options: Vec<HierarchyOption> = navigator.get_current_level_nodes()
                .iter()
                .map(|node| HierarchyOption {
                    key: node.key as i32,
                    label: node.label.clone().into(),
                    is_leaf: node.id.is_some(),
                })
                .collect();

            ui.set_hierarchy_options(slint::ModelRc::new(slint::VecModel::from(options)));
            ui.set_hierarchy_prompt(navigator.get_prompt().into());

            let breadcrumb = navigator.get_breadcrumb().join(" > ");
            ui.set_hierarchy_breadcrumb(breadcrumb.into());
        });
    }

    // Tracks the original pixel size of the currently displayed image.
    let image_dimensions = Rc::new(RefCell::new((1.0f32, 1.0f32)));
    let placeholder = placeholder_image();
    ui.set_image_source(placeholder.clone());

    // Populated only after a dataset is successfully loaded from disk.
    let dataset_state: Rc<RefCell<Option<DatasetState>>> = Rc::new(RefCell::new(None));

    // Attempt to load dataset from CLI arg if provided.
    let args: Vec<String> = std::env::args().collect();
    if let Some(ds_path) = args.get(1) {
        match load_dataset(Path::new(ds_path)) {
            Ok(state) => {
                let len = state.entries.len();
                let mut state = state;
                state.stored_annotations = vec![None; len];
                state.view_states = vec![None; len];
                state.completed_frames = vec![false; len];
                *dataset_state.borrow_mut() = Some(state);
            }
            Err(e) => {
                ui.set_status_text(format!("Dataset load error: {e}").into());
            }
        }
    } else {
        ui.set_status_text("No dataset provided (pass path as first arg)".into());
    }

    // Shared loader used by navigation callbacks to display image + annotations at a given index.
    let loader = {
        let annotations = annotations.clone();
        let ui_handle = ui.as_weak();
        let image_dimensions = image_dimensions.clone();
        let placeholder = placeholder.clone();
        let dataset_state = dataset_state.clone();
        let draw_state = draw_state.clone();
        Rc::new(move |index: usize| {
            let mut ds_opt = dataset_state.borrow_mut();
            let Some(ds) = ds_opt.as_mut() else { return; };
            if ds.stored_annotations.len() != ds.entries.len() {
                ds.stored_annotations.resize(ds.entries.len(), None);
            }
            if ds.view_states.len() != ds.entries.len() {
                ds.view_states.resize(ds.entries.len(), None);
            }
            if ds.completed_frames.len() != ds.entries.len() {
                ds.completed_frames.resize(ds.entries.len(), false);
            }
            if index >= ds.entries.len() {
                return;
            }

            ds.current_index = index;
            let entry = ds.entries[index].clone();

            let img_result = load_image_from_entry(&entry);
            let (image, img_size, status_msg) = match img_result {
                Ok(img) => {
                    let size = img.size();
                    (
                        img,
                        (size.width as f32, size.height as f32),
                        format!("Loaded {}", entry.image_path.display()),
                    )
                }
                Err(err) => {
                    let size = placeholder.size();
                    (
                        placeholder.clone(),
                        (size.width as f32, size.height as f32),
                        err,
                    )
                }
            };
            println!("{}", status_msg);

            *image_dimensions.borrow_mut() = img_size;

            // Load annotations from cache if available; otherwise from disk, then cache.
            let mut annotations_for_image = if let Some(cached) = ds.stored_annotations.get(index).and_then(|v| v.clone()) {
                cached
            } else {
                let anns = load_yolo_annotations(&entry, img_size, 1000);
                ds.stored_annotations[index] = Some(anns.clone());
                anns
            };

            // Clear selection when (re)loading to avoid stale references.
            for ann in annotations_for_image.iter_mut() {
                ann.selected = false;
            }

            replace_annotations(&annotations, annotations_for_image.clone());

            // Pick next id above existing annotations.
            draw_state.borrow_mut().next_id = next_id_from_annotations(&annotations_for_image, 2000);

            if let Some(ui) = ui_handle.upgrade() {
                ui.set_image_source(image);
                let fname = entry
                    .image_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("?");
                ui.set_current_image_name(fname.into());
                ui.set_dataset_position(format!("{} / {}", index + 1, ds.entries.len()).into());
                ui.set_status_text(status_msg.into());

                // Set completion status for this frame
                if index < ds.completed_frames.len() {
                    ui.set_frame_completed(ds.completed_frames[index]);
                } else {
                    ui.set_frame_completed(false);
                }

                // Apply view: prefer global (if same-ish size), else per-image cache, else reset.
                if let (Some(gv), Some(last_size)) =
                    (ds.global_view.clone(), ds.last_view_image_size)
                {
                    if sizes_close(last_size, img_size, 2.0) {
                        apply_view_state(&ui, &gv);
                        ds.view_states[index] = Some(gv.clone());
                        println!(
                            "Applied global view to index {}: pan=({}, {}), zoom={}",
                            index, gv.pan_x, gv.pan_y, gv.zoom
                        );
                        return;
                    }
                }

                if let Some(vs) = ds.view_states.get(index).and_then(|v| v.clone()) {
                    apply_view_state(&ui, &vs);
                    println!(
                        "Applied cached view to index {}: pan=({}, {}), zoom={}",
                        index, vs.pan_x, vs.pan_y, vs.zoom
                    );
                } else {
                    ui.invoke_reset_view();
                    let vs = get_view_state(&ui);
                    ds.view_states[index] = Some(vs.clone());
                    println!(
                        "Initial reset view for index {}: pan=({}, {}), zoom={}",
                        index, vs.pan_x, vs.pan_y, vs.zoom
                    );
                }
            }
        })
    };

    // Load first image if dataset present
    (loader)(0);

    // Selection callbacks (extracted to callbacks/selection.rs)
    callbacks::selection::setup_selection_callbacks(
        &ui,
        annotations.clone(),
        undo_history.clone(),
    );
    // Dataset navigation callbacks (extracted to callbacks/navigation.rs)
    callbacks::navigation::setup_navigation_callbacks(
        &ui,
        loader.clone(),
        dataset_state.clone(),
        annotations.clone(),
        image_dimensions.clone(),
    );

    // Track global view changes (pan/zoom) to reuse across images
    {
        let ds_state = dataset_state.clone();
        let image_dimensions = image_dimensions.clone();
        ui.on_view_changed(move |px, py, z| {
            if let Ok(mut ds_opt) = ds_state.try_borrow_mut() {
                if let Some(ds) = ds_opt.as_mut() {
                    ds.global_view = Some(ViewState { pan_x: px, pan_y: py, zoom: z });
                    ds.last_view_image_size = Some(*image_dimensions.borrow());
                }
            }
        });
    }

    ui.on_log_debug(move |msg| {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug_output.log")
        {
            let _ = writeln!(file, "{}", msg);
        }
    });
    // Drawing callbacks (extracted to callbacks/drawing.rs)
    callbacks::drawing::setup_drawing_callbacks(
        &ui,
        draw_state.clone(),
        annotations.clone(),
        undo_history.clone(),
    );

    // Delete annotation callback (for Q+click)
    let ui_handle = ui.as_weak();
    let annotations_handle = annotations.clone();
    let undo_history_ref = undo_history.clone();
    // Q + click: remove the topmost annotation under the cursor.
    ui.on_delete_annotation_at(move |x, y| {
        // Push current state to undo history before deletion
        undo_history_ref.borrow_mut().push(snapshot_annotations(&annotations_handle));

        let count = annotations_handle.row_count();
        for i in (0..count).rev() {
            // Reverse to get topmost first
            if let Some(ann) = annotations_handle.row_data(i) {
                if ann.state == "Rejected" {
                    continue;
                }
                // Check if point is inside this annotation
                let inside = if ann.r#type.as_str() == "point" {
                    // For points, use a small hit radius (10 pixels)
                    let dx = x - ann.x;
                    let dy = y - ann.y;
                    (dx * dx + dy * dy).sqrt() < 10.0
                } else {
                    // For boxes, check if inside bounds
                    x >= ann.x && x <= ann.x + ann.width && y >= ann.y && y <= ann.y + ann.height
                };

                if inside {
                    let mut rejected = ann;
                    rejected.state = "Rejected".into();
                    rejected.selected = false;
                    annotations_handle.set_row_data(i, rejected);
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_status_text("Annotation deleted".into());
                    }
                    break;
                }
            }
        }
    });

    // Delete annotation by index (for double-click)
    let ui_handle = ui.as_weak();
    let annotations_handle = annotations.clone();
    let undo_history_ref = undo_history.clone();
    ui.on_delete_annotation(move |index| {
        // Push current state to undo history before deletion
        undo_history_ref.borrow_mut().push(snapshot_annotations(&annotations_handle));

        if let Some(mut ann) = annotations_handle.row_data(index as usize) {
            ann.state = "Rejected".into();
            ann.selected = false;
            annotations_handle.set_row_data(index as usize, ann);
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text("Annotation deleted (double-click)".into());
            }
        }
    });

    // Classify annotation (for digit+click)
    let ui_handle = ui.as_weak();
    let annotations_handle = annotations.clone();
    let undo_history_ref = undo_history.clone();
    ui.on_classify_at(move |x, y, new_class| {
        // Push current state to undo history before classification
        undo_history_ref.borrow_mut().push(snapshot_annotations(&annotations_handle));

        // Find annotation at this position and update its class
        let count = annotations_handle.row_count();
        for i in (0..count).rev() {
            // Reverse to get topmost first
            if let Some(mut ann) = annotations_handle.row_data(i) {
                if ann.state == "Rejected" {
                    continue;
                }
                // Check if point is inside this annotation
                let inside = if ann.r#type.as_str() == "point" {
                    // For points, use a small hit radius (10 pixels)
                    let dx = x - ann.x;
                    let dy = y - ann.y;
                    (dx * dx + dy * dy).sqrt() < 10.0
                } else {
                    // For boxes, check if inside bounds
                    x >= ann.x && x <= ann.x + ann.width && y >= ann.y && y <= ann.y + ann.height
                };

                if inside {
                    ann.class = new_class;
                    if ann.state == "Pending" {
                        ann.state = "Accepted".into();
                    }
                    annotations_handle.set_row_data(i, ann);
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_status_text(
                            format!("Annotation reclassified to {}", new_class).into(),
                        );
                    }
                    break;
                }
            }
        }
    });

    // Classify currently selected annotation(s) when a digit key is pressed
    let ui_handle = ui.as_weak();
    let annotations_handle = annotations.clone();
    let undo_history_ref = undo_history.clone();
    ui.on_classify_selected(move |new_class| {
        // Push current state to undo history before classification
        undo_history_ref.borrow_mut().push(snapshot_annotations(&annotations_handle));

        let mut updated = false;
        let count = annotations_handle.row_count();
        for i in 0..count {
            if let Some(mut ann) = annotations_handle.row_data(i) {
                if ann.selected && ann.state != "Rejected" {
                    ann.class = new_class;
                    if ann.state == "Pending" {
                        ann.state = "Accepted".into();
                    }
                    annotations_handle.set_row_data(i, ann);
                    updated = true;
                }
            }
        }

        if updated {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text(
                    format!("Selected annotation set to class {}", new_class).into(),
                );
            }
        }
    });

    // Undo action (Ctrl+Z)
    {
        let undo_history_ref = undo_history.clone();
        let annotations_ref = annotations.clone();
        let ui_handle = ui.as_weak();

        ui.on_undo_action(move || {
            let current = snapshot_annotations(&annotations_ref);
            if let Some(previous) = undo_history_ref.borrow_mut().undo(current) {
                replace_annotations(&annotations_ref, previous);
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text("Undo".into());
                }
            } else if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text("Nothing to undo".into());
            }
        });
    }

    // Redo action (Ctrl+Shift+Z or Ctrl+Y)
    {
        let undo_history_ref = undo_history.clone();
        let annotations_ref = annotations.clone();
        let ui_handle = ui.as_weak();

        ui.on_redo_action(move || {
            let current = snapshot_annotations(&annotations_ref);
            if let Some(next) = undo_history_ref.borrow_mut().redo(current) {
                replace_annotations(&annotations_ref, next);
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text("Redo".into());
                }
            } else if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text("Nothing to redo".into());
            }
        });
    }

    // Copy annotation (Ctrl+C)
    {
        let clipboard_ref = clipboard.clone();
        let annotations_ref = annotations.clone();
        let ui_handle = ui.as_weak();

        ui.on_copy_annotation(move || {
            let count = annotations_ref.row_count();
            let mut copied_annotations = Vec::new();

            // Find all selected annotations
            for i in 0..count {
                if let Some(ann) = annotations_ref.row_data(i) {
                    if ann.selected {
                        copied_annotations.push(ann.clone());
                    }
                }
            }

            if copied_annotations.is_empty() {
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text("No annotation selected to copy".into());
                }
            } else {
                *clipboard_ref.borrow_mut() = copied_annotations.clone();
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text(format!("Copied {} annotation(s)", copied_annotations.len()).into());
                }
            }
        });
    }

    // Paste annotation (Ctrl+V)
    {
        let clipboard_ref = clipboard.clone();
        let annotations_ref = annotations.clone();
        let undo_history_ref = undo_history.clone();
        let ui_handle = ui.as_weak();

        ui.on_paste_annotation(move || {
            let copied_anns = clipboard_ref.borrow().clone();

            if copied_anns.is_empty() {
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text("No annotation to paste".into());
                }
                return;
            }

            // Push undo history before adding annotations
            let snapshot = snapshot_annotations(&annotations_ref);
            undo_history_ref.borrow_mut().push(snapshot);

            // Get starting ID for new annotations
            let existing: Vec<_> = (0..annotations_ref.row_count())
                .filter_map(|i| annotations_ref.row_data(i))
                .collect();
            let mut next_id = next_id_from_annotations(&existing, 1);

            // Offset by 5% in both directions
            let offset_x = 0.05;
            let offset_y = 0.05;

            // Paste all copied annotations
            for copied_ann in copied_anns.iter() {
                let mut new_ann = copied_ann.clone();
                new_ann.id = next_id;
                new_ann.x += offset_x;
                new_ann.y += offset_y;
                new_ann.selected = false; // Don't select the pasted annotations

                annotations_ref.push(new_ann);
                next_id += 1;
            }

            if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text(format!("Pasted {} annotation(s)", copied_anns.len()).into());
            }
        });
    }

    // Smart auto-resize using Sobel edge detection
    let annotations_handle = annotations.clone();
    let ui_handle = ui.as_weak();
    let image_dimensions_for_auto = image_dimensions.clone();
    let ds_state_for_auto = dataset_state.clone();
    ui.on_auto_resize_annotation(move |img_x, img_y, _gesture_kind| {
        let count = annotations_handle.row_count();
        let mut target_index: Option<usize> = None;

        // Find topmost bbox containing the click
        for i in (0..count).rev() {
            if let Some(ann) = annotations_handle.row_data(i) {
                if ann.state == "Rejected" {
                    continue;
                }
                let is_box = ann.r#type.as_str() == "bbox" || ann.r#type.as_str() == "rbbox";
                let inside = img_x >= ann.x
                    && img_x <= ann.x + ann.width
                    && img_y >= ann.y
                    && img_y <= ann.y + ann.height;

                if is_box && inside {
                    target_index = Some(i);
                    break;
                }
            }
        }

        if let Some(idx) = target_index {
            if let Some(mut ann) = annotations_handle.row_data(idx) {
                // Get current image path from dataset state
                let image_path = if let Ok(ds_opt) = ds_state_for_auto.try_borrow() {
                    if let Some(ds) = ds_opt.as_ref() {
                        if ds.current_index < ds.entries.len() {
                            Some(ds.entries[ds.current_index].image_path.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(path) = image_path {
                    let bbox = (ann.x, ann.y, ann.width, ann.height);
                    let img_size = *image_dimensions_for_auto.borrow();

                    if let Some((new_x, new_y, new_w, new_h)) =
                        auto_resize::smart_auto_resize(&path, bbox, img_size)
                    {
                        ann.x = new_x;
                        ann.y = new_y;
                        ann.width = new_w;
                        ann.height = new_h;
                        if ann.state == "Pending" {
                            ann.state = "Accepted".into();
                        }
                        annotations_handle.set_row_data(idx, ann);

                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text("Smart auto-resize applied".into());
                        }
                    } else if let Some(ui) = ui_handle.upgrade() {
                        ui.set_status_text("Auto-resize: failed to process".into());
                    }
                } else if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text("Auto-resize: image path not available".into());
                }
            }
        } else if let Some(ui) = ui_handle.upgrade() {
            ui.set_status_text("Auto-resize: no annotation under cursor".into());
        }
    });

    // Polygon creation callbacks: collect vertices while S is held, then commit to an annotation.
    let ui_handle = ui.as_weak();
    let draw_state_handle = draw_state.clone();
    ui.on_add_polygon_vertex(move |x, y| {
        let mut state = draw_state_handle.borrow_mut();
        state.polygon_vertices.push((x, y));

        // Update preview string and path
        if let Some(ui) = ui_handle.upgrade() {
            let vertices_str = state
                .polygon_vertices
                .iter()
                .map(|(vx, vy)| format!("{},{}", vx, vy))
                .collect::<Vec<_>>()
                .join(";");
            ui.set_polygon_preview_vertices(vertices_str.into());

            // Generate preview path commands (don't close the path with Z)
            let preview_path = if state.polygon_vertices.len() >= 2 {
                let mut commands = format!(
                    "M {} {}",
                    state.polygon_vertices[0].0, state.polygon_vertices[0].1
                );
                for vertex in state.polygon_vertices.iter().skip(1) {
                    commands.push_str(&format!(" L {} {}", vertex.0, vertex.1));
                }
                commands // Don't close with Z - polygon is still being drawn
            } else if state.polygon_vertices.len() == 1 {
                // Single vertex - just show a point (MoveTo with no lines)
                format!(
                    "M {} {}",
                    state.polygon_vertices[0].0, state.polygon_vertices[0].1
                )
            } else {
                String::new()
            };
            ui.set_polygon_preview_path(preview_path.into());

            ui.set_status_text(
                format!(
                    "Polygon: {} vertices (hold S, release S or Tab/Enter to finish)",
                    state.polygon_vertices.len()
                )
                .into(),
            );
        }
        println!(
            "Vertex added at ({:.1}, {:.1}), total: {}",
            x,
            y,
            state.polygon_vertices.len()
        );
    });

    let ui_handle = ui.as_weak();
    let annotations_handle = annotations.clone();
    let draw_state_handle = draw_state.clone();
    ui.on_finish_polygon(move || {
        let mut state = draw_state_handle.borrow_mut();

        // Need at least 3 vertices for a polygon
        if state.polygon_vertices.len() >= 3 {
            if let Some(ui) = ui_handle.upgrade() {
                let class = ui.get_current_class();

                // Convert vertices to string format
                let vertices_str = state
                    .polygon_vertices
                    .iter()
                    .map(|(x, y)| format!("{},{}", x, y))
                    .collect::<Vec<_>>()
                    .join(";");

                // Calculate bounding box for x, y, width, height
                let xs: Vec<f32> = state.polygon_vertices.iter().map(|(x, _)| *x).collect();
                let ys: Vec<f32> = state.polygon_vertices.iter().map(|(_, y)| *y).collect();
                let min_x = xs.iter().cloned().fold(f32::INFINITY, f32::min);
                let max_x = xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                let min_y = ys.iter().cloned().fold(f32::INFINITY, f32::min);
                let max_y = ys.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

                // Parse vertices for rendering
                let polygon_verts = parse_vertices(&vertices_str);

                // Generate SVG path commands for edge rendering
                let path_commands = generate_path_commands(&state.polygon_vertices);

                annotations_handle.push(Annotation {
                    id: state.next_id,
                    r#type: "polygon".into(),
                    x: min_x,
                    y: min_y,
                    width: max_x - min_x,
                    height: max_y - min_y,
                    rotation: 0.0,
                    selected: false,
                    class,
                    state: "Manual".into(),
                    vertices: vertices_str.clone().into(),
                    polygon_vertices: std::rc::Rc::new(slint::VecModel::from(polygon_verts)).into(),
                    polygon_path_commands: path_commands.into(),
                });
                state.next_id += 1;
                println!(
                    "Polygon created with {} vertices: {}",
                    state.polygon_vertices.len(),
                    vertices_str
                );
                ui.set_status_text(
                    format!(
                        "Polygon created with {} vertices",
                        state.polygon_vertices.len()
                    )
                    .into(),
                );
            }
        }

        // Clear polygon state
        state.polygon_vertices.clear();
        if let Some(ui) = ui_handle.upgrade() {
            ui.set_polygon_preview_vertices("".into());
            ui.set_polygon_preview_path("".into());
            ui.set_polygon_mode_active(false);
            ui.set_current_tool("Neutral".into());
        }
    });

    let ui_handle = ui.as_weak();
    let draw_state_handle = draw_state.clone();
    ui.on_cancel_polygon(move || {
        let mut state = draw_state_handle.borrow_mut();
        state.polygon_vertices.clear();

        if let Some(ui) = ui_handle.upgrade() {
            ui.set_polygon_preview_vertices("".into());
            ui.set_polygon_preview_path("".into());
            ui.set_polygon_mode_active(false);
            ui.set_current_tool("Neutral".into());
            ui.set_status_text("Polygon cancelled".into());
        }
    });

    // Resize callbacks
    let annotations_handle = annotations.clone();
    let resize_state_handle = resize_state.clone();
    // When a resize handle is grabbed, remember original bounds so deltas can be applied.
    ui.on_start_resize(move |index, handle_type| {
        if let Some(ann) = annotations_handle.row_data(index as usize) {
            if ann.state == "Rejected" {
                return;
            }
            let mut state = resize_state_handle.borrow_mut();
            state.annotation_index = index as usize;
            state.handle_type = handle_type.to_string();
            state.original_x = ann.x;
            state.original_y = ann.y;
            state.original_width = ann.width;
            state.original_height = ann.height;
            println!(
                "Start resize: index={}, handle={}, bounds=({:.1}, {:.1}, {:.1}, {:.1})",
                index, handle_type, ann.x, ann.y, ann.width, ann.height
            );
        }
    });

    let annotations_handle = annotations.clone();
    let resize_state_handle = resize_state.clone();
    ui.on_update_resize(move |mouse_x, mouse_y| {
        let state = resize_state_handle.borrow();
        let index = state.annotation_index;

        if let Some(mut ann) = annotations_handle.row_data(index) {
            if ann.state == "Rejected" {
                return;
            }
            let handle = state.handle_type.as_str();

            // Calculate new bounds based on handle type
            match handle {
                "corner-tl" => {
                    // Top-left corner: opposite corner (bottom-right) is fixed
                    let fixed_x = state.original_x + state.original_width;
                    let fixed_y = state.original_y + state.original_height;
                    ann.x = mouse_x.min(fixed_x);
                    ann.y = mouse_y.min(fixed_y);
                    ann.width = (fixed_x - ann.x).abs();
                    ann.height = (fixed_y - ann.y).abs();
                }
                "corner-tr" => {
                    // Top-right corner: opposite corner (bottom-left) is fixed
                    let fixed_x = state.original_x;
                    let fixed_y = state.original_y + state.original_height;
                    ann.x = mouse_x.min(fixed_x);
                    ann.y = mouse_y.min(fixed_y);
                    ann.width = (mouse_x - fixed_x).abs();
                    ann.height = (fixed_y - ann.y).abs();
                }
                "corner-bl" => {
                    // Bottom-left corner: opposite corner (top-right) is fixed
                    let fixed_x = state.original_x + state.original_width;
                    let fixed_y = state.original_y;
                    ann.x = mouse_x.min(fixed_x);
                    ann.y = mouse_y.min(fixed_y);
                    ann.width = (fixed_x - ann.x).abs();
                    ann.height = (mouse_y - fixed_y).abs();
                }
                "corner-br" => {
                    // Bottom-right corner: opposite corner (top-left) is fixed
                    let fixed_x = state.original_x;
                    let fixed_y = state.original_y;
                    ann.x = mouse_x.min(fixed_x);
                    ann.y = mouse_y.min(fixed_y);
                    ann.width = (mouse_x - fixed_x).abs();
                    ann.height = (mouse_y - fixed_y).abs();
                }
                "edge-t" => {
                    // Top edge: bottom is fixed
                    let fixed_y = state.original_y + state.original_height;
                    ann.y = mouse_y.min(fixed_y);
                    ann.height = (fixed_y - ann.y).abs();
                }
                "edge-r" => {
                    // Right edge: left is fixed
                    let fixed_x = state.original_x;
                    ann.width = (mouse_x - fixed_x).max(1.0); // Ensure minimum width
                }
                "edge-b" => {
                    // Bottom edge: top is fixed
                    let fixed_y = state.original_y;
                    ann.height = (mouse_y - fixed_y).max(1.0); // Ensure minimum height
                }
                "edge-l" => {
                    // Left edge: right is fixed
                    let fixed_x = state.original_x + state.original_width;
                    ann.x = mouse_x.min(fixed_x);
                    ann.width = (fixed_x - ann.x).abs();
                }
                _ => {}
            }

            if ann.state == "Pending" {
                ann.state = "Accepted".into();
            }
            annotations_handle.set_row_data(index, ann);
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_finish_resize(move || {
        if let Some(ui) = ui_handle.upgrade() {
            ui.set_status_text("Resize complete".into());
        }
        println!("Resize finished");
    });

    // Global view change tracking for persistence
    {
        let ds_state = dataset_state.clone();
        let image_dimensions = image_dimensions.clone();
        ui.on_view_changed(move |px, py, z| {
            if let Ok(mut ds_opt) = ds_state.try_borrow_mut() {
                if let Some(ds) = ds_opt.as_mut() {
                    ds.global_view = Some(ViewState { pan_x: px, pan_y: py, zoom: z });
                    ds.last_view_image_size = Some(*image_dimensions.borrow());
                }
            }
        });
    }

    // Save dataset callback (Ctrl/Cmd+S)
    {
        let ds_state = dataset_state.clone();
        let annotations_model = annotations.clone();
        let image_dimensions = image_dimensions.clone();
        let ui_handle = ui.as_weak();
        // manual save via Ctrl/Cmd+S
        ui.on_save_dataset(move || {
            if let (Ok(mut ds_opt), Some(ui)) = (ds_state.try_borrow_mut(), ui_handle.upgrade()) {
                if let Some(ds) = ds_opt.as_mut() {
                    // ensure current image state is cached
                    save_current_state(ds, &annotations_model, &ui, *image_dimensions.borrow());
                    match save_all(ds) {
                        Ok(_) => ui.set_status_text("Save successful".into()),
                        Err(e) => ui.set_status_text(format!("Save failed: {e}").into()),
                    }
                }
            }
        });
    }

    // Toggle frame completion (F key)
    {
        let ds_state = dataset_state.clone();
        let ui_handle = ui.as_weak();
        ui.on_toggle_frame_completion(move || {
            if let (Ok(mut ds_opt), Some(ui)) = (ds_state.try_borrow_mut(), ui_handle.upgrade()) {
                if let Some(ds) = ds_opt.as_mut() {
                    let idx = ds.current_index;
                    if idx < ds.completed_frames.len() {
                        ds.completed_frames[idx] = !ds.completed_frames[idx];
                        ui.set_frame_completed(ds.completed_frames[idx]);
                        let status = if ds.completed_frames[idx] { "✓ Frame marked as complete" } else { "Frame marked as incomplete" };
                        ui.set_status_text(status.into());
                    }
                }
            }
        });
    }

    // Phase 4: Open existing dataset
    {
        let ds_state = dataset_state.clone();
        let loader = loader.clone();
        let ui_handle = ui.as_weak();
        ui.on_open_dataset(move || {
            // Use file dialog to select dataset JSON
            let file = rfd::FileDialog::new()
                .add_filter("Dataset JSON", &["json"])
                .set_title("Open Dataset")
                .pick_file();

            if let Some(path) = file {
                match load_dataset(&path) {
                    Ok(state) => {
                        let len = state.entries.len();
                        let mut state = state;
                        state.stored_annotations = vec![None; len];
                        state.view_states = vec![None; len];
                        state.completed_frames = vec![false; len];
                        *ds_state.borrow_mut() = Some(state);

                        // Load first image
                        loader(0);

                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text(format!("Loaded dataset: {}", path.display()).into());
                        }
                    }
                    Err(e) => {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text(format!("Failed to load dataset: {e}").into());
                        }
                    }
                }
            }
        });
    }

    // Phase 4: Create new dataset from folder
    {
        let ds_state = dataset_state.clone();
        let loader = loader.clone();
        let ui_handle = ui.as_weak();
        ui.on_new_dataset(move || {
            // Use folder dialog to select image folder
            let folder = rfd::FileDialog::new()
                .set_title("Select Folder with Images")
                .pick_folder();

            if let Some(folder_path) = folder {
                match create_dataset_from_folder(&folder_path) {
                    Ok(manifest_path) => {
                        match load_dataset(&manifest_path) {
                            Ok(state) => {
                                let len = state.entries.len();
                                let mut state = state;
                                state.stored_annotations = vec![None; len];
                                state.view_states = vec![None; len];
                                state.completed_frames = vec![false; len];
                                *ds_state.borrow_mut() = Some(state);

                                // Load first image
                                loader(0);

                                if let Some(ui) = ui_handle.upgrade() {
                                    ui.set_status_text(format!("Created new dataset with {} images", len).into());
                                }
                            }
                            Err(e) => {
                                if let Some(ui) = ui_handle.upgrade() {
                                    ui.set_status_text(format!("Failed to load new dataset: {e}").into());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text(format!("Failed to create dataset: {e}").into());
                        }
                    }
                }
            }
        });
    }

    // Phase 5: Export as COCO JSON
    {
        let ds_state = dataset_state.clone();
        let classes_ref = classes.clone();
        let ui_handle = ui.as_weak();
        ui.on_export_coco(move || {
            // Pick folder to export to
            let folder = rfd::FileDialog::new()
                .set_title("Select Export Folder")
                .pick_folder();

            if let Some(export_folder) = folder {
                if let Ok(ds_opt) = ds_state.try_borrow() {
                    if let Some(ds) = ds_opt.as_ref() {
                        // Create COCO dataset
                        let mut coco = export::coco::CocoDataset::new();

                        // Add categories from class config
                        for class_def in &classes_ref.borrow().classes {
                            coco.add_category(class_def.id, class_def.name.clone());
                        }

                        let mut ann_id = 1;

                        // Export each image
                        for (img_idx, entry) in ds.entries.iter().enumerate() {
                            let filename = entry.image_path.file_name()
                                .and_then(|f| f.to_str())
                                .unwrap_or("unknown.png")
                                .to_string();

                            // Load image to get dimensions
                            let (width, height) = if let Ok(img) = load_image_from_entry(entry) {
                                let size = img.size();
                                (size.width as i32, size.height as i32)
                            } else {
                                (640, 480) // fallback
                            };

                            // Add image entry
                            coco.images.push(export::coco::CocoImage {
                                id: (img_idx + 1) as i32,
                                width,
                                height,
                                file_name: filename,
                            });

                            // Get annotations for this image
                            if let Some(Some(annotations)) = ds.stored_annotations.get(img_idx) {
                                for ann in annotations {
                                    // Convert annotation based on type
                                    let (bbox_opt, segmentation_opt, area_opt) = match ann.r#type.as_str() {
                                        "bbox" | "rbbox" => {
                                            let bbox = [
                                                ann.x as f64,
                                                ann.y as f64,
                                                ann.width as f64,
                                                ann.height as f64,
                                            ];
                                            let area = ann.width as f64 * ann.height as f64;
                                            (Some(bbox), None, Some(area))
                                        }
                                        "point" => {
                                            // Point as small bbox
                                            let bbox = [ann.x as f64, ann.y as f64, 1.0, 1.0];
                                            (Some(bbox), None, Some(1.0))
                                        }
                                        "polygon" => {
                                            // Parse polygon vertices
                                            let verts: Vec<f64> = ann.vertices.as_str()
                                                .split(',')
                                                .filter_map(|s| s.trim().parse().ok())
                                                .collect();
                                            let area = if verts.len() >= 6 {
                                                // Calculate area using shoelace formula
                                                let mut a = 0.0;
                                                for i in 0..verts.len() / 2 {
                                                    let j = (i + 1) % (verts.len() / 2);
                                                    a += verts[i * 2] * verts[j * 2 + 1];
                                                    a -= verts[j * 2] * verts[i * 2 + 1];
                                                }
                                                (a / 2.0).abs()
                                            } else {
                                                0.0
                                            };
                                            (None, Some(vec![verts]), Some(area))
                                        }
                                        _ => continue,
                                    };

                                    coco.annotations.push(export::coco::CocoAnnotation {
                                        id: ann_id,
                                        image_id: (img_idx + 1) as i32,
                                        category_id: ann.class,
                                        bbox: bbox_opt,
                                        segmentation: segmentation_opt,
                                        area: area_opt,
                                        iscrowd: 0,
                                    });
                                    ann_id += 1;
                                }
                            }
                        }

                        // Save COCO JSON
                        let coco_path = export_folder.join("annotations.json");
                        match coco.save(&coco_path) {
                            Ok(_) => {
                                if let Some(ui) = ui_handle.upgrade() {
                                    ui.set_status_text(format!(
                                        "Exported {} images with {} annotations to COCO JSON",
                                        coco.images.len(),
                                        coco.annotations.len()
                                    ).into());
                                }
                            }
                            Err(e) => {
                                if let Some(ui) = ui_handle.upgrade() {
                                    ui.set_status_text(format!("Export failed: {e}").into());
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    // Phase 5: Export as Pascal VOC XML
    {
        let ds_state = dataset_state.clone();
        let classes_ref = classes.clone();
        let ui_handle = ui.as_weak();
        ui.on_export_voc(move || {
            // Pick folder to export to
            let folder = rfd::FileDialog::new()
                .set_title("Select Export Folder")
                .pick_folder();

            if let Some(export_folder) = folder {
                if let Ok(ds_opt) = ds_state.try_borrow() {
                    if let Some(ds) = ds_opt.as_ref() {
                        let mut total_files = 0;
                        let mut total_annotations = 0;

                        // Export each image as separate XML file
                        for (img_idx, entry) in ds.entries.iter().enumerate() {
                            let filename = entry.image_path.file_name()
                                .and_then(|f| f.to_str())
                                .unwrap_or("unknown.png")
                                .to_string();

                            // Load image to get dimensions
                            let (width, height) = if let Ok(img) = load_image_from_entry(entry) {
                                let size = img.size();
                                (size.width as i32, size.height as i32)
                            } else {
                                (640, 480) // fallback
                            };

                            let mut voc_ann = export::voc::VocAnnotation::new(filename.clone(), width, height);

                            // Get annotations for this image
                            let mut has_annotations = false;
                            if let Some(Some(annotations)) = ds.stored_annotations.get(img_idx) {
                                for ann in annotations {
                                    // Only export bounding boxes for VOC
                                    if ann.r#type.as_str() == "bbox" || ann.r#type.as_str() == "rbbox" {
                                        let class_name = classes::get_class_name(&classes_ref.borrow(), ann.class);
                                        let xmin = ann.x as i32;
                                        let ymin = ann.y as i32;
                                        let xmax = (ann.x + ann.width) as i32;
                                        let ymax = (ann.y + ann.height) as i32;
                                        voc_ann.add_object(class_name, xmin, ymin, xmax, ymax);
                                        has_annotations = true;
                                        total_annotations += 1;
                                    }
                                }
                            }

                            // Save XML file (even if no annotations)
                            if has_annotations || true {  // Export all files
                                let xml_filename = Path::new(&filename).with_extension("xml");
                                let xml_path = export_folder.join(xml_filename);
                                if let Err(e) = voc_ann.save(&xml_path) {
                                    if let Some(ui) = ui_handle.upgrade() {
                                        ui.set_status_text(format!("Export failed: {e}").into());
                                    }
                                    return;
                                }
                                total_files += 1;
                            }
                        }

                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text(format!(
                                "Exported {} XML files with {} annotations to Pascal VOC",
                                total_files,
                                total_annotations
                            ).into());
                        }
                    }
                }
            }
        });
    }

    // Auto-save timer every 5 seconds
    {
        let ds_state = dataset_state.clone();
        let annotations_model = annotations.clone();
        let image_dimensions = image_dimensions.clone();
        let ui_handle = ui.as_weak();
        slint::Timer::default().start(slint::TimerMode::Repeated, std::time::Duration::from_secs(5), move || {
            if let (Ok(mut ds_opt), Some(ui)) = (ds_state.try_borrow_mut(), ui_handle.upgrade()) {
                if let Some(ds) = ds_opt.as_mut() {
                    save_current_state(ds, &annotations_model, &ui, *image_dimensions.borrow());
                    if let Err(e) = save_all(ds) {
                        ui.set_status_text(format!("Autosave failed: {e}").into());
                    }
                }
            }
        });
    }

    ui.run()
}
