// P3-040: Many annotations performance
// Test: Load thousands of annotations and test interaction
// Expected: Pan/zoom remain smooth, no crashes

slint::include_modules!();
use slint::Model;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let image_path = std::path::PathBuf::from("/Users/jacobvaught/RUST/images/wide.jpeg");
    let image = slint::Image::load_from_path(&image_path).expect("Failed to load test image");
    ui.set_image_source(image);

    // Create a large number of annotations
    let annotation_count = 1000;
    println!("Creating {} annotations...", annotation_count);

    let mut annotation_list = Vec::new();
    for i in 0..annotation_count {
        let x = ((i % 100) * 50) as f32;
        let y = ((i / 100) * 50) as f32;
        annotation_list.push(Annotation {
            id: i as i32,
            r#type: "bbox".into(),
            x,
            y,
            width: 40.0,
            height: 40.0,
            rotation: 0.0,
            selected: false,
            class: 1,
            vertices: "".into(),
            polygon_vertices: Default::default(),
            polygon_path_commands: "".into(),
        });
    }

    let annotations = std::rc::Rc::new(slint::VecModel::from(annotation_list));
    ui.set_annotations(annotations.clone().into());
    println!("✓ {} annotations loaded", annotation_count);

    let annotations_handle = annotations.clone();
    ui.on_select_annotation(move |index| {
        let count = annotations_handle.row_count();
        for i in 0..count {
            let mut data = annotations_handle.row_data(i).unwrap();
            data.selected = i == index as usize;
            annotations_handle.set_row_data(i, data);
        }
        if index % 100 == 0 {
            println!("Selected annotation {}", index);
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

    println!("=== P3-040: Performance with Many Annotations ===");
    println!("Loaded {} annotations in a grid pattern", annotation_count);
    println!("Instructions:");
    println!("1. Try panning the image - should remain interactive");
    println!("2. Try zooming in and out - frame rate may drop but should be usable");
    println!("3. Try selecting annotations by clicking");
    println!("4. VERIFY: No crashes or out-of-memory errors");
    println!("5. VERIFY: Interaction remains responsive enough for use");
    println!("6. Monitor CPU/memory usage if possible");
    println!("=================================================");
    println!("NOTE: If performance is poor, reduce annotation_count in the test file");

    ui.run()
}
