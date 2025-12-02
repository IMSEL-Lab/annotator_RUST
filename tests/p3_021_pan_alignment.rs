// P3-021: Pan keeps overlay locked to image
// Test: Pan and verify annotations move with image
// Expected: Annotations stay aligned, no drift or lag

slint::include_modules!();
use slint::Model;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let image_path = std::path::PathBuf::from("/Users/jacobvaught/RUST/images/wide.jpeg");
    let image = slint::Image::load_from_path(&image_path).expect("Failed to load test image");
    ui.set_image_source(image);

    // Create annotations that will be easy to track during panning
    let annotations = std::rc::Rc::new(slint::VecModel::from(vec![
        Annotation {
            id: 1,
            r#type: "bbox".into(),
            x: 300.0,
            y: 200.0,
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
            x: 400.0,
            y: 275.0,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
            selected: false,
            class: 1,
            vertices: "".into(),
            polygon_vertices: Default::default(),
            polygon_path_commands: "".into(),
        },
        Annotation {
            id: 3,
            r#type: "rbbox".into(),
            x: 600.0,
            y: 300.0,
            width: 180.0,
            height: 100.0,
            rotation: 45.0,
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

    println!("=== P3-021: Pan Alignment ===");
    println!("Instructions:");
    println!("1. Note the position of the red box and point at center");
    println!("2. Click and drag to pan the image in various directions");
    println!("3. VERIFY: The box and point move EXACTLY with the image");
    println!("4. VERIFY: No lag, drift, or offset appears during panning");
    println!("5. The rotated box should also stay perfectly aligned");
    println!("6. Try rapid panning and smooth panning");
    println!("==============================");

    ui.run()
}
