// P3-020: Overlay alignment at initial view
// Test: Annotations align correctly at fit-to-view
// Expected: Boxes and points align with image features

slint::include_modules!();
use slint::Model;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let image_path = std::path::PathBuf::from("/Users/jacobvaught/RUST/images/wide.jpeg");
    let image = slint::Image::load_from_path(&image_path).expect("Failed to load test image");
    ui.set_image_source(image);

    // Create annotations at specific image coordinates
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
            r#type: "rbbox".into(),
            x: 400.0,
            y: 300.0,
            width: 250.0,
            height: 120.0,
            rotation: 30.0,
            selected: false,
            class: 1,
            vertices: "".into(),
            polygon_vertices: Default::default(),
            polygon_path_commands: "".into(),
        },
        Annotation {
            id: 3,
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
        Annotation {
            id: 4,
            r#type: "bbox".into(),
            x: 800.0,
            y: 400.0,
            width: 150.0,
            height: 100.0,
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

    println!("=== P3-020: Initial Alignment ===");
    println!("Instructions:");
    println!("1. At initial fit-to-view, verify all annotations are visible");
    println!("2. Check that boxes, rotated boxes, and points appear correctly");
    println!("3. Verify no visible offset or scale errors");
    println!("4. Annotations should be at their specified image coordinates");
    println!("==================================");

    ui.run()
}
