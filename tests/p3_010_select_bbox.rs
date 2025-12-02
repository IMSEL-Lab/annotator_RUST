// P3-010: Select a single axis-aligned bbox
// Test: Click clearly inside the bbox, away from edges
// Expected: Exactly that bbox becomes selected with visual change

slint::include_modules!();
use slint::Model;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // Load test image
    let image_path = std::path::PathBuf::from("/Users/jacobvaught/RUST/images/wide.jpeg");
    let image = slint::Image::load_from_path(&image_path).expect("Failed to load test image");
    ui.set_image_source(image);

    // Create test scene with a single bbox
    let annotations = std::rc::Rc::new(slint::VecModel::from(vec![
        Annotation {
            id: 1,
            r#type: "bbox".into(),
            x: 300.0,
            y: 200.0,
            width: 300.0,
            height: 200.0,
            rotation: 0.0,
            selected: false,
            class: 1,
            vertices: "".into(),
            polygon_vertices: Default::default(),
            polygon_path_commands: "".into(),
        },
    ]));
    ui.set_annotations(annotations.clone().into());

    // Setup selection callbacks
    let annotations_handle = annotations.clone();
    ui.on_select_annotation(move |index| {
        let count = annotations_handle.row_count();
        for i in 0..count {
            let mut data = annotations_handle.row_data(i).unwrap();
            data.selected = i == index as usize;
            annotations_handle.set_row_data(i, data);
        }
        println!("✓ Annotation {} selected", index);
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
        println!("✓ All annotations deselected");
    });

    println!("=== P3-010: Select Single BBox ===");
    println!("Instructions:");
    println!("1. Click inside the red rectangle (centered on screen)");
    println!("2. Verify the rectangle turns GREEN");
    println!("3. Click outside the rectangle");
    println!("4. Verify the rectangle turns back to RED");
    println!("=====================================");

    ui.run()
}
