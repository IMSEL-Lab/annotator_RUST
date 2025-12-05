//! Auto-resize callback using edge detection.
//!
//! Handles: auto_resize_annotation (smart bbox resizing)

use crate::state::DatasetState;
use crate::{auto_resize, Annotation, AppWindow};
use slint::{ComponentHandle, Model};
use std::cell::RefCell;
use std::rc::Rc;

/// Sets up the auto-resize annotation callback on the UI.
pub fn setup_auto_resize_callback(
    ui: &AppWindow,
    annotations: Rc<slint::VecModel<Annotation>>,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    image_dimensions: Rc<RefCell<(f32, f32)>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_auto_resize_annotation(move |img_x, img_y, _gesture_kind| {
        let count = annotations.row_count();
        let mut target_index: Option<usize> = None;

        // Find topmost bbox containing the click
        for i in (0..count).rev() {
            if let Some(ann) = annotations.row_data(i) {
                if ann.state == "Rejected" {
                    continue;
                }
                let is_box = ann.r#type.as_str() == "bbox" || ann.r#type.as_str() == "rbbox";
                let inside = img_x >= ann.x
                    && img_x <= ann.x + ann.width
                    && img_y >= ann.y
                    && img_y <= ann.y + ann.height;

                if is_box && inside {
                    target_index = Some(i);
                    break;
                }
            }
        }

        if let Some(idx) = target_index {
            if let Some(mut ann) = annotations.row_data(idx) {
                let image_path = if let Ok(ds_opt) = dataset_state.try_borrow() {
                    if let Some(ds) = ds_opt.as_ref() {
                        if ds.current_index < ds.entries.len() {
                            Some(ds.entries[ds.current_index].image_path.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(path) = image_path {
                    let bbox = (ann.x, ann.y, ann.width, ann.height);
                    let img_size = *image_dimensions.borrow();

                    if let Some((new_x, new_y, new_w, new_h)) =
                        auto_resize::smart_auto_resize(&path, bbox, img_size)
                    {
                        ann.x = new_x;
                        ann.y = new_y;
                        ann.width = new_w;
                        ann.height = new_h;
                        if ann.state == "Pending" {
                            ann.state = "Accepted".into();
                        }
                        annotations.set_row_data(idx, ann);

                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_status_text("Smart auto-resize applied".into());
                        }
                    } else if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_text("Auto-resize: failed to process".into());
                    }
                } else if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_text("Auto-resize: image path not available".into());
                }
            }
        } else if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_text("Auto-resize: no annotation under cursor".into());
        }
    });
}
