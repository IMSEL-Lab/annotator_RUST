// Test helper functions for creating test scenarios
use slint::Model;

slint::include_modules!();

/// Create a standard test annotation (axis-aligned bbox)
pub fn create_bbox(id: i32, x: f32, y: f32, width: f32, height: f32, selected: bool) -> Annotation {
    Annotation {
        id,
        r#type: "bbox".into(),
        x,
        y,
        width,
        height,
        rotation: 0.0,
        selected,
    }
}

/// Create a rotated bounding box
pub fn create_rbbox(id: i32, x: f32, y: f32, width: f32, height: f32, rotation: f32, selected: bool) -> Annotation {
    Annotation {
        id,
        r#type: "rbbox".into(),
        x,
        y,
        width,
        height,
        rotation,
        selected,
    }
}

/// Create a center point annotation
pub fn create_point(id: i32, x: f32, y: f32, selected: bool) -> Annotation {
    Annotation {
        id,
        r#type: "point".into(),
        x,
        y,
        width: 0.0,
        height: 0.0,
        rotation: 0.0,
        selected,
    }
}

/// Create a test scene with basic annotations
pub fn create_basic_scene() -> std::rc::Rc<slint::VecModel<Annotation>> {
    std::rc::Rc::new(slint::VecModel::from(vec![
        create_bbox(1, 100.0, 100.0, 200.0, 150.0, false),
        create_point(2, 600.0, 200.0, false),
    ]))
}

/// Create a test scene with overlapping boxes for selection testing
pub fn create_overlapping_boxes_scene() -> std::rc::Rc<slint::VecModel<Annotation>> {
    std::rc::Rc::new(slint::VecModel::from(vec![
        create_bbox(1, 100.0, 100.0, 200.0, 200.0, false),
        create_bbox(2, 150.0, 150.0, 200.0, 200.0, false), // Overlaps with box 1
    ]))
}

/// Create a test scene with rotated boxes
pub fn create_rotated_boxes_scene() -> std::rc::Rc<slint::VecModel<Annotation>> {
    std::rc::Rc::new(slint::VecModel::from(vec![
        create_bbox(1, 100.0, 100.0, 200.0, 150.0, false),
        create_rbbox(2, 400.0, 300.0, 250.0, 120.0, 30.0, false),
        create_rbbox(3, 700.0, 400.0, 180.0, 100.0, 45.0, false),
    ]))
}

/// Create a large scene for performance testing
pub fn create_large_scene(count: usize) -> std::rc::Rc<slint::VecModel<Annotation>> {
    let mut annotations = Vec::new();

    for i in 0..count {
        let x = ((i % 100) * 50) as f32;
        let y = ((i / 100) * 50) as f32;
        annotations.push(create_bbox(i as i32, x, y, 40.0, 40.0, false));
    }

    std::rc::Rc::new(slint::VecModel::from(annotations))
}

/// Helper to count selected annotations
pub fn count_selected(annotations: &std::rc::Rc<slint::VecModel<Annotation>>) -> usize {
    let mut count = 0;
    for i in 0..annotations.row_count() {
        if let Some(ann) = annotations.row_data(i) {
            if ann.selected {
                count += 1;
            }
        }
    }
    count
}

/// Helper to get selected annotation ID (if any)
pub fn get_selected_id(annotations: &std::rc::Rc<slint::VecModel<Annotation>>) -> Option<i32> {
    for i in 0..annotations.row_count() {
        if let Some(ann) = annotations.row_data(i) {
            if ann.selected {
                return Some(ann.id);
            }
        }
    }
    None
}

/// Helper to verify no annotations are selected
pub fn verify_none_selected(annotations: &std::rc::Rc<slint::VecModel<Annotation>>) -> bool {
    count_selected(annotations) == 0
}
