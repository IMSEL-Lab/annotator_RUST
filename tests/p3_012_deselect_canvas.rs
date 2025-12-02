// P3-012: Deselect by clicking empty canvas
// Test: With annotation selected, click empty area
// Expected: Previously selected annotation becomes unselected

slint::include_modules!();
use slint::Model;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let image_path = std::path::PathBuf::from("/Users/jacobvaught/RUST/images/wide.jpeg");
    let image = slint::Image::load_from_path(&image_path).expect("Failed to load test image");
    ui.set_image_source(image);

    // Create test scene with pre-selected annotation
    let annotations = std::rc::Rc::new(slint::VecModel::from(vec![
        Annotation {
            id: 1,
            r#type: "bbox".into(),
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 150.0,
            rotation: 0.0,
            selected: true,
            class: 1,
            vertices: "".into(),
            polygon_vertices: Default::default(),
            polygon_path_commands: "".into(), // Pre-selected
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
        let mut deselected_any = false;
        for i in 0..count {
            let mut data = annotations_handle.row_data(i).unwrap();
            if data.selected {
                data.selected = false;
                annotations_handle.set_row_data(i, data);
                deselected_any = true;
            }
        }
        if deselected_any {
            println!("✓ PASS: Deselect triggered, annotations cleared");
        }
    });

    println!("=== P3-012: Deselect by Canvas Click ===");
    println!("Instructions:");
    println!("1. Note the GREEN rectangle (it's pre-selected)");
    println!("2. Click on empty canvas area (not on any annotation)");
    println!("3. Verify the rectangle turns RED (unselected)");
    println!("4. Try selecting it again, then deselect multiple times");
    println!("=========================================");

    ui.run()
}
