slint::include_modules!();

mod config;
mod classes;
mod export;

use serde::Deserialize;
use serde::Serialize;
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
        stored_annotations: Vec::new(),
        view_states: Vec::new(),
        global_view: None,
        last_view_image_size: None,
    })
}

fn create_dataset_from_folder(folder: &Path) -> Result<PathBuf, String> {
    // Scan folder for image files
    let extensions = ["png", "jpg", "jpeg", "bmp", "gif"];
    let mut image_files = Vec::new();

    let entries = fs::read_dir(folder)
        .map_err(|e| format!("Failed to read folder: {e}"))?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if extensions.contains(&ext_str.to_lowercase().as_str()) {
                            if let Some(filename) = path.file_name() {
                                image_files.push(filename.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    if image_files.is_empty() {
        return Err("No image files found in folder".into());
    }

    // Sort for consistent ordering
    image_files.sort();

    // Create manifest entries
    let manifest_entries: Vec<DatasetFileEntry> = image_files
        .into_iter()
        .map(|img| DatasetFileEntry {
            image: img.clone(),
            labels: Some(Path::new(&img).with_extension("txt").to_string_lossy().to_string()),
        })
        .collect();

    let manifest = DatasetFile {
        images: manifest_entries,
    };

    // Save manifest.json in the folder
    let manifest_path = folder.join("manifest.json");
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {e}"))?;
    fs::write(&manifest_path, json)
        .map_err(|e| format!("Failed to write manifest: {e}"))?;

    Ok(manifest_path)
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
    // Prefer persisted state file if present
    let state_path = state_path_for(entry);
    if let Ok(text) = fs::read_to_string(&state_path) {
        if let Ok(stored) = serde_json::from_str::<Vec<StoredAnnotation>>(&text) {
            return stored
                .into_iter()
                .map(|s| Annotation {
                    id: s.id,
                    r#type: s.r#type.into(),
                    x: s.x,
                    y: s.y,
                    width: s.width,
                    height: s.height,
                    rotation: s.rotation,
                    selected: s.selected,
                    class: s.class,
                    state: s.state.into(),
                    vertices: s.vertices.into(),
                    polygon_vertices: Default::default(),
                    polygon_path_commands: "".into(),
                })
                .collect();
        }
    }

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
            state: "Pending".into(),
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

fn snapshot_annotations(model: &slint::VecModel<Annotation>) -> Vec<Annotation> {
    let mut out = Vec::with_capacity(model.row_count());
    for i in 0..model.row_count() {
        if let Some(ann) = model.row_data(i) {
            out.push(ann);
        }
    }
    out
}

fn next_id_from_annotations(anns: &[Annotation], default_start: i32) -> i32 {
    anns.iter().map(|a| a.id).max().map(|m| m + 1).unwrap_or(default_start)
}

fn get_view_state(ui: &AppWindow) -> ViewState {
    ViewState {
        pan_x: ui.get_view_pan_x(),
        pan_y: ui.get_view_pan_y(),
        zoom: ui.get_view_zoom().max(0.1),
    }
}

fn apply_view_state(ui: &AppWindow, vs: &ViewState) {
    let safe_zoom = if vs.zoom <= 0.0 || !vs.zoom.is_finite() { 1.0 } else { vs.zoom };
    ui.set_view_pan_x(vs.pan_x);
    ui.set_view_pan_y(vs.pan_y);
    ui.set_view_zoom(safe_zoom);

    // Also push into global view change callback to keep downstream state coherent
    ui.invoke_view_changed(vs.pan_x, vs.pan_y, safe_zoom);
}

fn sizes_close(a: (f32, f32), b: (f32, f32), tolerance: f32) -> bool {
    (a.0 - b.0).abs() <= tolerance && (a.1 - b.1).abs() <= tolerance
}

fn label_path_for(entry: &DatasetEntry) -> PathBuf {
    entry
        .labels_path
        .clone()
        .unwrap_or_else(|| entry.image_path.with_extension("txt"))
}

fn state_path_for(entry: &DatasetEntry) -> PathBuf {
    label_path_for(entry).with_extension("state.json")
}

fn save_current_state(
    ds: &mut DatasetState,
    annotations: &slint::VecModel<Annotation>,
    ui: &AppWindow,
    img_size: (f32, f32),
) {
    let idx = ds.current_index;
    if idx >= ds.entries.len() {
        return;
    }
    ds.stored_annotations[idx] = Some(snapshot_annotations(annotations));
    ds.view_states[idx] = Some(get_view_state(ui));
    ds.global_view = ds.view_states[idx].clone();
    ds.last_view_image_size = Some(img_size);
}

fn ann_to_stored(a: &Annotation) -> StoredAnnotation {
    StoredAnnotation {
        id: a.id,
        r#type: a.r#type.to_string(),
        x: a.x,
        y: a.y,
        width: a.width,
        height: a.height,
        rotation: a.rotation,
        selected: false,
        class: a.class,
        state: a.state.to_string(),
        vertices: a.vertices.to_string(),
    }
}

fn save_all(ds: &DatasetState) -> Result<(), String> {
    for (idx, entry) in ds.entries.iter().enumerate() {
        let anns = ds
            .stored_annotations
            .get(idx)
            .and_then(|v| v.clone())
            .unwrap_or_default();

        // Write YOLO labels (bbox/rbbox only, non-rejected)
        let label_path = label_path_for(entry);
        if let Some(parent) = label_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Label dir create: {e}"))?;
        }

        let mut yolo_lines = Vec::new();
        // Load image size to normalize
        let img_size = slint::Image::load_from_path(&entry.image_path)
            .map(|img| img.size())
            .map(|s| (s.width as f32, s.height as f32))
            .unwrap_or((1.0, 1.0));

        for a in anns.iter() {
            if a.state == "Rejected" {
                continue;
            }
            if a.r#type == "bbox" || a.r#type == "rbbox" {
                let cx = (a.x + a.width / 2.0) / img_size.0;
                let cy = (a.y + a.height / 2.0) / img_size.1;
                let w = (a.width / img_size.0).clamp(0.0, 1.0);
                let h = (a.height / img_size.1).clamp(0.0, 1.0);
                let cls = (a.class - 1).max(0);
                yolo_lines.push(format!("{cls} {cx} {cy} {w} {h}"));
            }
        }
        std::fs::write(&label_path, yolo_lines.join("\n"))
            .map_err(|e| format!("Write labels {}: {e}", label_path.display()))?;

        // Write state file with all annotations
        let state_path = state_path_for(entry);
        let stored: Vec<StoredAnnotation> = anns.iter().map(ann_to_stored).collect();
        let json = serde_json::to_string_pretty(&stored)
            .map_err(|e| format!("Serialize state: {e}"))?;
        if let Some(parent) = state_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("State dir create: {e}"))?;
        }
        std::fs::write(&state_path, json)
            .map_err(|e| format!("Write state {}: {e}", state_path.display()))?;
    }
    Ok(())
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

#[derive(Debug, Serialize, Deserialize)]
struct DatasetFileEntry {
    image: String,
    labels: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatasetFile {
    images: Vec<DatasetFileEntry>,
}

#[derive(Debug, Clone)]
struct DatasetEntry {
    image_path: PathBuf,
    labels_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
struct ViewState {
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredAnnotation {
    id: i32,
    r#type: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rotation: f32,
    selected: bool,
    class: i32,
    state: String,
    vertices: String,
}

#[derive(Debug, Clone)]
struct DatasetState {
    entries: Vec<DatasetEntry>,
    current_index: usize,
    stored_annotations: Vec<Option<Vec<Annotation>>>,
    view_states: Vec<Option<ViewState>>,
    global_view: Option<ViewState>,
    last_view_image_size: Option<(f32, f32)>,
}

// Parse hex color string to Slint Color
fn parse_color(hex: &str) -> Option<slint::Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(slint::Color::from_rgb_u8(r, g, b))
    } else {
        None
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // Load configuration
    let config = Rc::new(RefCell::new(config::load_config()));

    // Load class definitions
    let class_config_path = config.borrow().classes.config_file.clone();
    let classes = Rc::new(RefCell::new(classes::load_classes(class_config_path.as_deref())));

    // Apply initial theme from config
    let _theme_name = config.borrow().appearance.theme.clone();
    // Theme will be set via callback later if needed
    // For now, it defaults to dark theme in the Slint code

    // Populate class items for the sidebar
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
                let len = state.entries.len();
                let mut state = state;
                state.stored_annotations = vec![None; len];
                state.view_states = vec![None; len];
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
    let annotations_for_save = annotations.clone();
    let ui_for_save = ui.as_weak();
    let image_dimensions_next = image_dimensions.clone();
    ui.on_next_image(move || {
        // Drop the immutable borrow before calling the loader (which mutably borrows)
        let next_idx = {
            let mut ds_ref = ds_state_next.borrow_mut();
            let Some(ds) = ds_ref.as_mut() else { return; };
            if ds.entries.is_empty() {
                return;
            }

            if let Some(ui) = ui_for_save.upgrade() {
                save_current_state(ds, &annotations_for_save, &ui, *image_dimensions_next.borrow());
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
    let annotations_for_save = annotations.clone();
    let ui_for_save = ui.as_weak();
    let image_dimensions_prev = image_dimensions.clone();
    ui.on_prev_image(move || {
        let prev_idx = {
            let mut ds_ref = ds_state_prev.borrow_mut();
            let Some(ds) = ds_ref.as_mut() else { return; };
            if ds.entries.is_empty() {
                return;
            }

            if let Some(ui) = ui_for_save.upgrade() {
                save_current_state(ds, &annotations_for_save, &ui, *image_dimensions_prev.borrow());
            }

            if ds.current_index == 0 {
                0
            } else {
                ds.current_index - 1
            }
        };

        loader_prev(prev_idx);
    });

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
                        state: "Manual".into(),
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
                    state: "Manual".into(),
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
    ui.on_delete_annotation(move |index| {
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
    ui.on_classify_at(move |x, y, new_class| {
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
    ui.on_classify_selected(move |new_class| {
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

    // Auto-resize (stub) callbacks
    let annotations_handle = annotations.clone();
    let ui_handle = ui.as_weak();
    let image_dimensions_for_auto = image_dimensions.clone();
    // Auto-resize stub: adjust the first box under cursor using heuristics (pinch/edge gestures).
    ui.on_auto_resize_annotation(move |img_x, img_y, gesture_kind| {
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
                if let Some((new_x, new_y, new_w, new_h)) =
                    auto_resize_stub(&ann, gesture_kind.as_str(), *image_dimensions_for_auto.borrow())
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
