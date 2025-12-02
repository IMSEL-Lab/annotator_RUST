// P3-013: Topmost selection when shapes overlap
// Test: Click in overlapping region
// Expected: Topmost annotation (last in list) becomes selected

slint::include_modules!();
use slint::Model;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let image_path = std::path::PathBuf::from("/Users/jacobvaught/RUST/images/wide.jpeg");
    let image = slint::Image::load_from_path(&image_path).expect("Failed to load test image");
    ui.set_image_source(image);

    // Create overlapping boxes - Box 2 is rendered on top of Box 1
    let annotations = std::rc::Rc::new(slint::VecModel::from(vec![
        Annotation {
            id: 1,
            r#type: "bbox".into(),
            x: 200.0,
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
        Annotation {
            id: 2,
            r#type: "bbox".into(),
            x: 250.0,
            y: 250.0,
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

    let annotations_handle = annotations.clone();
    ui.on_select_annotation(move |index| {
        let count = annotations_handle.row_count();
        for i in 0..count {
            let mut data = annotations_handle.row_data(i).unwrap();
            data.selected = i == index as usize;
            annotations_handle.set_row_data(i, data);
        }
        let selected = annotations_handle.row_data(index as usize).unwrap();
        println!("✓ Annotation index {} (ID: {}) selected", index, selected.id);
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
        println!("✓ All deselected");
    });

    println!("=== P3-013: Overlapping Selection ===");
    println!("Instructions:");
    println!("1. You should see two overlapping red rectangles");
    println!("2. Click in the overlapping region (center-right area)");
    println!("3. EXPECTED: Box 2 (the top/right box) should be selected");
    println!("4. Click in the non-overlapping area of Box 1 (left side)");
    println!("5. EXPECTED: Box 1 should be selected");
    println!("6. Verify consistent behavior across multiple clicks");
    println!("======================================");

    ui.run()
}
