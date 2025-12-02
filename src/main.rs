slint::include_modules!();

use slint::Model;
use std::cell::RefCell;
use std::rc::Rc;

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

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let draw_state = Rc::new(RefCell::new(DrawState::new()));
    let resize_state = Rc::new(RefCell::new(ResizeState::new()));

    // Load test image
    let image_path = std::path::PathBuf::from("/Users/jacobvaught/RUST/images/wide.jpeg");
    let image = slint::Image::load_from_path(&image_path).expect("Failed to load test image");
    ui.set_image_source(image);

    // Initialize Annotations
    let annotations = std::rc::Rc::new(slint::VecModel::from(vec![
        Annotation {
            id: 1,
            r#type: "bbox".into(),
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 150.0,
            rotation: 0.0,
            selected: false,
            class: 1,
            vertices: "".into(),
            polygon_vertices: Default::default(),
            polygon_path_commands: "".into(),
        },
        Annotation {
            id: 2,
            r#type: "point".into(),
            x: 600.0,
            y: 200.0,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
            selected: false,
            class: 1,
            vertices: "".into(),
            polygon_vertices: Default::default(),
            polygon_path_commands: "".into(),
        },
    ]));
    ui.set_annotations(annotations.clone().into());

    // Selection Callbacks
    let annotations_handle = annotations.clone();
    ui.on_select_annotation(move |index| {
        let count = annotations_handle.row_count();
        for i in 0..count {
            let mut data = annotations_handle.row_data(i).unwrap();
            data.selected = i == index as usize;
            annotations_handle.set_row_data(i, data);
        }
    });

    let annotations_handle = annotations.clone();
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
    ui.on_delete_annotation_at(move |x, y| {
        // Find annotation at this position and delete it
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

    // Polygon creation callbacks
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
