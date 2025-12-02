// P3-011: Select a center point
// Test: Click near the center point marker within a reasonable hit radius
// Expected: The center point is marked selected with visual indication

slint::include_modules!();
use slint::Model;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let image_path = std::path::PathBuf::from("/Users/jacobvaught/RUST/images/wide.jpeg");
    let image = slint::Image::load_from_path(&image_path).expect("Failed to load test image");
    ui.set_image_source(image);

    // Create test scene with multiple points
    let annotations = std::rc::Rc::new(slint::VecModel::from(vec![
        Annotation {
            id: 1,
            r#type: "point".into(),
            x: 300.0,
            y: 200.0,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
            selected: false,
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
        },
        Annotation {
            id: 3,
            r#type: "point".into(),
            x: 450.0,
            y: 400.0,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
            selected: false,
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
        println!("✓ Point {} selected", index);
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
        println!("✓ All points deselected");
    });

    println!("=== P3-011: Select Center Point ===");
    println!("Instructions:");
    println!("1. Click on each red dot (there are 3)");
    println!("2. Verify each dot turns GREEN when clicked");
    println!("3. Try clicking near but not on a dot - it should NOT select");
    println!("4. Click empty space to deselect");
    println!("=====================================");

    ui.run()
}
