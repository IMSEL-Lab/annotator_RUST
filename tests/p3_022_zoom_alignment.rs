// P3-022: Zoom keeps overlay locked to image
// Test: Zoom in/out and verify annotations scale properly
// Expected: Annotations scale with image, no parallax

slint::include_modules!();

mod test_helpers;
use test_helpers::*;
use slint::Model;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let image_path = std::path::PathBuf::from("/Users/jacobvaught/RUST/images/wide.jpeg");
    let image = slint::Image::load_from_path(&image_path).expect("Failed to load test image");
    ui.set_image_source(image);

    // Create annotations with known sizes
    let annotations = std::rc::Rc::new(slint::VecModel::from(vec![
        create_bbox(1, 200.0, 150.0, 300.0, 200.0, false),
        create_rbbox(2, 550.0, 350.0, 200.0, 150.0, 30.0, false),
        create_point(3, 400.0, 250.0, false),
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
        println!("Selected annotation {}", index);
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

    println!("=== P3-022: Zoom Alignment ===");
    println!("Instructions:");
    println!("1. Use mouse wheel to zoom IN several steps");
    println!("2. VERIFY: Annotations scale proportionally with the image");
    println!("3. VERIFY: Annotations stay over the same image features");
    println!("4. Zoom OUT several steps");
    println!("5. VERIFY: No parallax or offset during zoom");
    println!("6. CRITICAL: Test the rotated box (ID 2) - it should scale AND rotate correctly");
    println!("7. The point should remain visible at all zoom levels");
    println!("===============================");

    ui.run()
}
