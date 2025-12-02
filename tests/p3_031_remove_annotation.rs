// P3-031: Remove annotation programmatically
// Test: Dynamically remove annotations
// Expected: Removed shape disappears, others remain

slint::include_modules!();
use slint::Model;
use std::time::Duration;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let image_path = std::path::PathBuf::from("/Users/jacobvaught/RUST/images/wide.jpeg");
    let image = slint::Image::load_from_path(&image_path).expect("Failed to load test image");
    ui.set_image_source(image);

    // Start with multiple annotations
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
            y: 200.0,
            width: 200.0,
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
            y: 300.0,
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
            x: 700.0,
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

    // Schedule annotations to be removed via event loop
    let annotations_clone = annotations.clone();
    let timer1 = slint::Timer::default();
    timer1.start(slint::TimerMode::SingleShot, Duration::from_secs(2), move || {
        println!("Removing annotation at index 2 (the point)...");
        annotations_clone.remove(2);
    });

    let annotations_clone = annotations.clone();
    let timer2 = slint::Timer::default();
    timer2.start(slint::TimerMode::SingleShot, Duration::from_secs(4), move || {
        println!("Removing annotation at index 1 (the rotated box)...");
        annotations_clone.remove(1);
    });

    let _timer3 = slint::Timer::default();
    _timer3.start(slint::TimerMode::SingleShot, Duration::from_secs(6), move || {
        println!("All removals complete. 2 boxes should remain.");
    });

    println!("=== P3-031: Remove Annotation Programmatically ===");
    println!("Instructions:");
    println!("1. App starts with 4 annotations (2 boxes, 1 rotated box, 1 point)");
    println!("2. After 2 seconds, the POINT will disappear");
    println!("3. After 4 seconds, the ROTATED BOX will disappear");
    println!("4. After 6 seconds, only 2 REGULAR BOXES should remain");
    println!("5. VERIFY: Removed shapes disappear completely");
    println!("6. VERIFY: Remaining shapes stay in correct positions");
    println!("===================================================");

    ui.run()
}
