// P3-030: Add annotation programmatically
// Test: Dynamically add a new annotation after startup
// Expected: New shape appears at correct position

slint::include_modules!();
use slint::Model;
use std::time::Duration;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let image_path = std::path::PathBuf::from("/Users/jacobvaught/RUST/images/wide.jpeg");
    let image = slint::Image::load_from_path(&image_path).expect("Failed to load test image");
    ui.set_image_source(image);

    // Start with initial annotations
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
        },
        Annotation {
            id: 2,
            r#type: "point".into(),
            x: 400.0,
            y: 200.0,
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

    // Schedule annotations to be added via event loop
    let annotations_clone = annotations.clone();
    let timer1 = slint::Timer::default();
    timer1.start(slint::TimerMode::SingleShot, Duration::from_secs(2), move || {
        println!("Adding rotated box annotation...");
        annotations_clone.push(Annotation {
            id: 3,
            r#type: "rbbox".into(),
            x: 600.0,
            y: 300.0,
            width: 180.0,
            height: 120.0,
            rotation: 45.0,
            selected: false,
        });
    });

    let annotations_clone = annotations.clone();
    let timer2 = slint::Timer::default();
    timer2.start(slint::TimerMode::SingleShot, Duration::from_secs(4), move || {
        println!("Adding another bbox annotation...");
        annotations_clone.push(Annotation {
            id: 4,
            r#type: "bbox".into(),
            x: 500.0,
            y: 400.0,
            width: 150.0,
            height: 100.0,
            rotation: 0.0,
            selected: false,
        });
    });

    let annotations_clone = annotations.clone();
    let timer3 = slint::Timer::default();
    timer3.start(slint::TimerMode::SingleShot, Duration::from_secs(6), move || {
        println!("Adding point annotation...");
        annotations_clone.push(Annotation {
            id: 5,
            r#type: "point".into(),
            x: 700.0,
            y: 250.0,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
            selected: false,
        });
    });

    println!("=== P3-030: Add Annotation Programmatically ===");
    println!("Instructions:");
    println!("1. App starts with 1 box and 1 point");
    println!("2. After 2 seconds, a ROTATED BOX will appear");
    println!("3. After 4 seconds, another REGULAR BOX will appear");
    println!("4. After 6 seconds, a POINT will appear");
    println!("5. VERIFY: Each new annotation appears at correct position");
    println!("6. VERIFY: Existing annotations remain unchanged");
    println!("================================================");

    ui.run()
}
