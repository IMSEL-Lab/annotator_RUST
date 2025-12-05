//! Dataset loading, saving, and management functions.

use crate::state::types::{
    DatasetEntry, DatasetFile, DatasetFileEntry, DatasetState, StoredAnnotation, ViewState,
};
use crate::{Annotation, AppWindow, PolygonVertex};
use slint::Model;
use std::fs;
use std::path::{Path, PathBuf};

/// Load a dataset from a manifest JSON file
pub fn load_dataset(path: &Path) -> Result<DatasetState, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read dataset: {e}"))?;
    let parsed: DatasetFile =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse dataset JSON: {e}"))?;

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
        completed_frames: Vec::new(),
    })
}

/// Create a new dataset manifest from a folder of images
pub fn create_dataset_from_folder(folder: &Path) -> Result<PathBuf, String> {
    // Scan folder for image files
    let extensions = ["png", "jpg", "jpeg", "bmp", "gif"];
    let mut image_files = Vec::new();

    let entries =
        fs::read_dir(folder).map_err(|e| format!("Failed to read folder: {e}"))?;

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
            labels: Some(
                Path::new(&img)
                    .with_extension("txt")
                    .to_string_lossy()
                    .to_string(),
            ),
        })
        .collect();

    let manifest = DatasetFile {
        images: manifest_entries,
    };

    // Save manifest.json in the folder
    let manifest_path = folder.join("manifest.json");
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {e}"))?;
    fs::write(&manifest_path, json).map_err(|e| format!("Failed to write manifest: {e}"))?;

    Ok(manifest_path)
}

/// Load an image from a dataset entry
pub fn load_image_from_entry(entry: &DatasetEntry) -> Result<slint::Image, String> {
    slint::Image::load_from_path(&entry.image_path)
        .map_err(|_| format!("Image not found: {}", entry.image_path.display()))
}

/// Load YOLO-format annotations for a dataset entry
pub fn load_yolo_annotations(
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
        let cls: i32 = parts[0].parse().unwrap_or(0) + 1; // shift to 1-based class IDs
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

/// Get the label file path for a dataset entry
pub fn label_path_for(entry: &DatasetEntry) -> PathBuf {
    entry
        .labels_path
        .clone()
        .unwrap_or_else(|| entry.image_path.with_extension("txt"))
}

/// Get the state file path for a dataset entry
pub fn state_path_for(entry: &DatasetEntry) -> PathBuf {
    label_path_for(entry).with_extension("state.json")
}

/// Save current state to the dataset
pub fn save_current_state(
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

/// Convert an Annotation to StoredAnnotation format
pub fn ann_to_stored(a: &Annotation) -> StoredAnnotation {
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

/// Save all dataset entries to disk
pub fn save_all(ds: &DatasetState) -> Result<(), String> {
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
        let json =
            serde_json::to_string_pretty(&stored).map_err(|e| format!("Serialize state: {e}"))?;
        if let Some(parent) = state_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("State dir create: {e}"))?;
        }
        std::fs::write(&state_path, json)
            .map_err(|e| format!("Write state {}: {e}", state_path.display()))?;
    }
    Ok(())
}

// ============================================================================
// Helper functions moved from main.rs
// ============================================================================

/// Get the current view state from the UI
pub fn get_view_state(ui: &AppWindow) -> ViewState {
    ViewState {
        pan_x: ui.get_view_pan_x(),
        pan_y: ui.get_view_pan_y(),
        zoom: ui.get_view_zoom().max(0.1),
    }
}

/// Apply a view state to the UI
pub fn apply_view_state(ui: &AppWindow, vs: &ViewState) {
    let safe_zoom = if vs.zoom <= 0.0 || !vs.zoom.is_finite() {
        1.0
    } else {
        vs.zoom
    };
    ui.set_view_pan_x(vs.pan_x);
    ui.set_view_pan_y(vs.pan_y);
    ui.set_view_zoom(safe_zoom);

    // Also push into global view change callback to keep downstream state coherent
    ui.invoke_view_changed(vs.pan_x, vs.pan_y, safe_zoom);
}

/// Check if two sizes are close within a tolerance
pub fn sizes_close(a: (f32, f32), b: (f32, f32), tolerance: f32) -> bool {
    (a.0 - b.0).abs() <= tolerance && (a.1 - b.1).abs() <= tolerance
}

/// Replace all annotations in a VecModel
pub fn replace_annotations(model: &slint::VecModel<Annotation>, anns: Vec<Annotation>) {
    for _ in (0..model.row_count()).rev() {
        model.remove(model.row_count() - 1);
    }
    for ann in anns {
        model.push(ann);
    }
}

/// Take a snapshot of all annotations in a VecModel
pub fn snapshot_annotations(model: &slint::VecModel<Annotation>) -> Vec<Annotation> {
    let mut out = Vec::with_capacity(model.row_count());
    for i in 0..model.row_count() {
        if let Some(ann) = model.row_data(i) {
            out.push(ann);
        }
    }
    out
}

/// Get the next available annotation ID from existing annotations
pub fn next_id_from_annotations(anns: &[Annotation], default_start: i32) -> i32 {
    anns.iter()
        .map(|a| a.id)
        .max()
        .map(|m| m + 1)
        .unwrap_or(default_start)
}

/// Parse a vertices string into PolygonVertex array
pub fn parse_vertices(vertices_str: &str) -> Vec<PolygonVertex> {
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

/// Generate SVG path commands from vertices
pub fn generate_path_commands(vertices: &[(f32, f32)]) -> String {
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
