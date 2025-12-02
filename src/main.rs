slint::include_modules!();

use serde::Deserialize;
use slint::{Model, SharedPixelBuffer};
use std::cell::RefCell;
use std::rc::Rc;
use std::path::{Path, PathBuf};
use std::fs;

/// Deterministic stub for auto-resize. Scales bbox about its center and clamps to image bounds.
fn auto_resize_stub(
    ann: &Annotation,
    gesture_kind: &str,
    image_size: (f32, f32),
) -> Option<(f32, f32, f32, f32)> {
    // Only bbox/rbbox are handled in this stub
    if ann.width <= 0.0 || ann.height <= 0.0 {
        return None;
    }

    let scale = match gesture_kind {
        "AClick" => 1.2,
        _ => 1.0,
    };

    let cx = ann.x + ann.width / 2.0;
    let cy = ann.y + ann.height / 2.0;

    let mut new_w = ann.width * scale;
    let mut new_h = ann.height * scale;

    let (img_w, img_h) = image_size;

    // If scaled box is larger than the image, clamp to image size
    if new_w > img_w {
        new_w = img_w;
    }
    if new_h > img_h {
        new_h = img_h;
    }

    let mut new_x = cx - new_w / 2.0;
    let mut new_y = cy - new_h / 2.0;

    // Clamp to keep box fully inside the image
    if new_x < 0.0 {
        new_x = 0.0;
    }
    if new_y < 0.0 {
        new_y = 0.0;
    }
    if new_x + new_w > img_w {
        new_x = (img_w - new_w).max(0.0);
    }
    if new_y + new_h > img_h {
        new_y = (img_h - new_h).max(0.0);
    }

    Some((new_x, new_y, new_w, new_h))
}

fn placeholder_image() -> slint::Image {
    let width = 64u32;
    let height = 64u32;
    let mut buffer = SharedPixelBuffer::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let v = if (x / 8 + y / 8) % 2 == 0 { 60 } else { 110 };
            let i = ((y * width + x) * 3) as usize;
            let data = buffer.make_mut_bytes();
            data[i] = v;
            data[i + 1] = v;
            data[i + 2] = v;
        }
    }
    slint::Image::from_rgb8(buffer)
}

fn load_dataset(path: &Path) -> Result<DatasetState, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read dataset: {e}"))?;
    let parsed: DatasetFile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse dataset JSON: {e}"))?;

    let base_dir = path.parent().unwrap_or(Path::new("."));
    let mut entries = Vec::new();
    for entry in parsed.images {
        let image_path = base_dir.join(entry.image);
        let labels_path = entry.labels.map(|lp| base_dir.join(lp));
        entries.push(DatasetEntry {
            image_path,
            labels_path,
        });
    }

    if entries.is_empty() {
        return Err("Dataset has no images".into());
    }

    Ok(DatasetState {
        entries,
        current_index: 0,
    })
}

fn load_image_from_entry(entry: &DatasetEntry) -> Result<slint::Image, String> {
    slint::Image::load_from_path(&entry.image_path)
        .map_err(|_| format!("Image not found: {}", entry.image_path.display()))
}

fn load_yolo_annotations(
    entry: &DatasetEntry,
    img_size: (f32, f32),
    next_id_start: i32,
) -> Vec<Annotation> {
    let mut anns = Vec::new();
    let Some(label_path) = entry.labels_path.as_ref() else {
        return anns;
    };
    let Ok(text) = fs::read_to_string(label_path) else {
        return anns;
    };

    for (idx, line) in text.lines().enumerate() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 5 {
            continue;
        }
        let cls: i32 = parts[0].parse().unwrap_or(0) + 1; // shift to 1-based class IDs used in UI
        let cx: f32 = parts[1].parse().unwrap_or(0.5);
        let cy: f32 = parts[2].parse().unwrap_or(0.5);
        let w: f32 = parts[3].parse().unwrap_or(0.0);
        let h: f32 = parts[4].parse().unwrap_or(0.0);

        let img_w = img_size.0;
        let img_h = img_size.1;

        let abs_w = w * img_w;
        let abs_h = h * img_h;
        let x = cx * img_w - abs_w / 2.0;
        let y = cy * img_h - abs_h / 2.0;

        anns.push(Annotation {
            id: next_id_start + idx as i32,
            r#type: "bbox".into(),
            x,
            y,
            width: abs_w,
            height: abs_h,
            rotation: 0.0,
            selected: false,
            class: cls,
            vertices: "".into(),
            polygon_vertices: Default::default(),
            polygon_path_commands: "".into(),
        });
    }
    anns
}

fn replace_annotations(model: &slint::VecModel<Annotation>, anns: Vec<Annotation>) {
    for _ in (0..model.row_count()).rev() {
        model.remove(model.row_count() - 1);
    }
    for ann in anns {
        model.push(ann);
    }
}

// Helper function to parse vertices string into PolygonVertex array
fn parse_vertices(vertices_str: &str) -> Vec<PolygonVertex> {
    vertices_str
        .split(';')
        .filter(|s| !s.is_empty())
        .filter_map(|pair| {
            let parts: Vec<&str> = pair.split(',').collect();
            if parts.len() == 2 {
                let x = parts[0].parse::<f32>().ok()?;
                let y = parts[1].parse::<f32>().ok()?;
                Some(PolygonVertex { x, y })
            } else {
                None
            }
        })
        .collect()
}

// Generate SVG path commands from vertices
fn generate_path_commands(vertices: &[(f32, f32)]) -> String {
    if vertices.is_empty() {
        return String::new();
    }

    let mut commands = format!("M {} {}", vertices[0].0, vertices[0].1);

    for vertex in vertices.iter().skip(1) {
        commands.push_str(&format!(" L {} {}", vertex.0, vertex.1));
    }

    commands.push_str(" Z"); // Close path
    commands
}

// Drawing state
struct DrawState {
    start_x: f32,
    start_y: f32,
    next_id: i32,
    polygon_vertices: Vec<(f32, f32)>, // Vertices for polygon being created
}

impl DrawState {
    fn new() -> Self {
        Self {
            start_x: 0.0,
            start_y: 0.0,
            next_id: 100, // Start from 100 to avoid conflicts with test data
            polygon_vertices: Vec::new(),
        }
    }
}

// Resize state
struct ResizeState {
    annotation_index: usize,
    handle_type: String, // "corner-tl", "corner-tr", "corner-bl", "corner-br", "edge-t", "edge-r", "edge-b", "edge-l"
    original_x: f32,
    original_y: f32,
    original_width: f32,
    original_height: f32,
}

impl ResizeState {
    fn new() -> Self {
        Self {
            annotation_index: 0,
            handle_type: String::new(),
            original_x: 0.0,
            original_y: 0.0,
            original_width: 0.0,
            original_height: 0.0,
        }
    }
}

#[derive(Debug, Deserialize)]
struct DatasetFileEntry {
    image: String,
    labels: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DatasetFile {
    images: Vec<DatasetFileEntry>,
}

#[derive(Debug, Clone)]
struct DatasetEntry {
    image_path: PathBuf,
    labels_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct DatasetState {
    entries: Vec<DatasetEntry>,
    current_index: usize,
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let draw_state = Rc::new(RefCell::new(DrawState::new()));
    let resize_state = Rc::new(RefCell::new(ResizeState::new()));
    let annotations = std::rc::Rc::new(slint::VecModel::from(Vec::<Annotation>::new()));
    ui.set_annotations(annotations.clone().into());

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

            *image_dimensions.borrow_mut() = img_size;

            let annotations_for_image = load_yolo_annotations(&entry, img_size, 1000);
            replace_annotations(&annotations, annotations_for_image);
            draw_state.borrow_mut().next_id = 2000;

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
            }
        })
    };

    // Load first image if dataset present
    (loader)(0);

    // Selection Callbacks
    let annotations_handle = annotations.clone();
    // Single-selection from the sidebar list; marks only the clicked row as selected.
    ui.on_select_annotation(move |index| {
        let count = annotations_handle.row_count();
        for i in 0..count {
            let mut data = annotations_handle.row_data(i).unwrap();
            data.selected = i == index as usize;
            annotations_handle.set_row_data(i, data);
        }
    });

    let annotations_handle = annotations.clone();
    // Clear selection when the canvas is clicked with no modifier.
    ui.on_deselect_all(move || {
        let count = annotations_handle.row_count();
        for i in 0..count {
            let mut data = annotations_handle.row_data(i).unwrap();
            if data.selected {
                data.selected = false;
                annotations_handle.set_row_data(i, data);
            }
        }
    });

    // Dataset navigation callbacks
    let loader_next = loader.clone();
    let ds_state_next = dataset_state.clone();
    ui.on_next_image(move || {
        // Drop the immutable borrow before calling the loader (which mutably borrows)
        let next_idx = {
            let ds_ref = ds_state_next.borrow();
            let Some(ds) = ds_ref.as_ref() else { return; };
            if ds.entries.is_empty() {
                return;
            }
            let mut idx = ds.current_index;
            if idx + 1 < ds.entries.len() {
                idx += 1;
            }
            idx
        };

        loader_next(next_idx);
    });

    let loader_prev = loader.clone();
    let ds_state_prev = dataset_state.clone();
    ui.on_prev_image(move || {
        let prev_idx = {
            let ds_ref = ds_state_prev.borrow();
            let Some(ds) = ds_ref.as_ref() else { return; };
            if ds.entries.is_empty() {
                return;
            }
            if ds.current_index == 0 {
                0
            } else {
                ds.current_index - 1
            }
        };

        loader_prev(prev_idx);
    });

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

    // Drawing callbacks
    let ui_handle = ui.as_weak();
    let draw_state_handle = draw_state.clone();
    // Begin box/point drawing: remember anchor and show live preview rectangle.
    ui.on_start_drawing(move |x, y| {
        let mut state = draw_state_handle.borrow_mut();
        state.start_x = x;
        state.start_y = y;

        if let Some(ui) = ui_handle.upgrade() {
            ui.set_show_preview(true);
            ui.set_preview_x(x);
            ui.set_preview_y(y);
            ui.set_preview_width(0.0);
            ui.set_preview_height(0.0);
        }
    });

    let ui_handle = ui.as_weak();
    let draw_state_handle = draw_state.clone();
    ui.on_update_drawing(move |x, y| {
        let state = draw_state_handle.borrow();

        if let Some(ui) = ui_handle.upgrade() {
            let min_x = state.start_x.min(x);
            let min_y = state.start_y.min(y);
            let width = (x - state.start_x).abs();
            let height = (y - state.start_y).abs();

            ui.set_preview_x(min_x);
            ui.set_preview_y(min_y);
            ui.set_preview_width(width);
            ui.set_preview_height(height);
        }
    });

    let ui_handle = ui.as_weak();
    let annotations_handle = annotations.clone();
    let draw_state_handle = draw_state.clone();
    // Finalize a bbox or point when the mouse button is released.
    ui.on_finish_drawing(move |x, y| {
        let mut state = draw_state_handle.borrow_mut();

        if let Some(ui) = ui_handle.upgrade() {
            ui.set_show_preview(false);

            let min_x = state.start_x.min(x);
            let min_y = state.start_y.min(y);
            let width = (x - state.start_x).abs();
            let height = (y - state.start_y).abs();

            let tool = ui.get_current_tool();
            let class = ui.get_current_class();

            if tool.as_str().starts_with("BBox") {
                // Create bbox annotation only if size is reasonable (at least 5 pixels)
                if width >= 5.0 && height >= 5.0 {
                    annotations_handle.push(Annotation {
                        id: state.next_id,
                        r#type: "bbox".into(),
                        x: min_x,
                        y: min_y,
                        width,
                        height,
                        rotation: 0.0,
                        selected: false,
                        class,
                        vertices: "".into(),
                        polygon_vertices: Default::default(),
                        polygon_path_commands: "".into(),
                    });
                    state.next_id += 1;
                }
            } else if tool.as_str().starts_with("Point") {
                // Create point annotation at click location (no minimum size)
                annotations_handle.push(Annotation {
                    id: state.next_id,
                    r#type: "point".into(),
                    x,
                    y,
                    width: 0.0,
                    height: 0.0,
                    rotation: 0.0,
                    selected: false,
                    class,
                    vertices: "".into(),
                    polygon_vertices: Default::default(),
                    polygon_path_commands: "".into(),
                });
                state.next_id += 1;
            }
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_cancel_drawing(move || {
        if let Some(ui) = ui_handle.upgrade() {
            ui.set_show_preview(false);
        }
    });

    // Delete annotation callback (for Q+click)
    let ui_handle = ui.as_weak();
    let annotations_handle = annotations.clone();
    // Q + click: remove the topmost annotation under the cursor.
    ui.on_delete_annotation_at(move |x, y| {
        let count = annotations_handle.row_count();
        for i in (0..count).rev() {
            // Reverse to get topmost first
            if let Some(ann) = annotations_handle.row_data(i) {
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
                    annotations_handle.remove(i);
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
    ui.on_delete_annotation(move |index| {
        annotations_handle.remove(index as usize);
        if let Some(ui) = ui_handle.upgrade() {
            ui.set_status_text("Annotation deleted (double-click)".into());
        }
    });

    // Classify annotation (for digit+click)
    let ui_handle = ui.as_weak();
    let annotations_handle = annotations.clone();
    ui.on_classify_at(move |x, y, new_class| {
        // Find annotation at this position and update its class
        let count = annotations_handle.row_count();
        for i in (0..count).rev() {
            // Reverse to get topmost first
            if let Some(mut ann) = annotations_handle.row_data(i) {
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
    ui.on_classify_selected(move |new_class| {
        let mut updated = false;
        let count = annotations_handle.row_count();
        for i in 0..count {
            if let Some(mut ann) = annotations_handle.row_data(i) {
                if ann.selected {
                    ann.class = new_class;
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

    // Auto-resize (stub) callbacks
    let annotations_handle = annotations.clone();
    let ui_handle = ui.as_weak();
    // Auto-resize stub: adjust the first box under cursor using heuristics (pinch/edge gestures).
    ui.on_auto_resize_annotation(move |img_x, img_y, gesture_kind| {
        let count = annotations_handle.row_count();
        let mut target_index: Option<usize> = None;

        // Find topmost bbox containing the click
        for i in (0..count).rev() {
            if let Some(ann) = annotations_handle.row_data(i) {
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
                if let Some((new_x, new_y, new_w, new_h)) =
                    auto_resize_stub(&ann, gesture_kind.as_str(), *image_dimensions.borrow())
                {
                    ann.x = new_x;
                    ann.y = new_y;
                    ann.width = new_w;
                    ann.height = new_h;
                    annotations_handle.set_row_data(idx, ann);

                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_status_text(
                            format!("Auto-resize applied ({})", gesture_kind).into(),
                        );
                    }
                } else if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text("Auto-resize: no change".into());
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

    ui.run()
}
