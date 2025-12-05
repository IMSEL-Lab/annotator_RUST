//! Resize callbacks for annotation resizing.
//!
//! Handles: start_resize, update_resize, finish_resize

use crate::state::ResizeState;
use crate::{Annotation, AppWindow};
use slint::{ComponentHandle, Model};
use std::cell::RefCell;
use std::rc::Rc;

/// Sets up all resize-related callbacks on the UI.
pub fn setup_resize_callbacks(
    ui: &AppWindow,
    resize_state: Rc<RefCell<ResizeState>>,
    annotations: Rc<slint::VecModel<Annotation>>,
) {
    setup_start_resize(ui, resize_state.clone(), annotations.clone());
    setup_update_resize(ui, resize_state, annotations);
    setup_finish_resize(ui);
}

fn setup_start_resize(
    ui: &AppWindow,
    resize_state: Rc<RefCell<ResizeState>>,
    annotations: Rc<slint::VecModel<Annotation>>,
) {
    ui.on_start_resize(move |index, handle_type| {
        if let Some(ann) = annotations.row_data(index as usize) {
            if ann.state == "Rejected" {
                return;
            }
            let mut state = resize_state.borrow_mut();
            state.annotation_index = index as usize;
            state.handle_type = handle_type.to_string();
            state.original_x = ann.x;
            state.original_y = ann.y;
            state.original_width = ann.width;
            state.original_height = ann.height;
            println!(
                "Start resize: index={}, handle={}, bounds=({:.1}, {:.1}, {:.1}, {:.1})",
                index, handle_type, ann.x, ann.y, ann.width, ann.height
            );
        }
    });
}

fn setup_update_resize(
    ui: &AppWindow,
    resize_state: Rc<RefCell<ResizeState>>,
    annotations: Rc<slint::VecModel<Annotation>>,
) {
    ui.on_update_resize(move |mouse_x, mouse_y| {
        let state = resize_state.borrow();
        let index = state.annotation_index;

        if let Some(mut ann) = annotations.row_data(index) {
            if ann.state == "Rejected" {
                return;
            }
            let handle = state.handle_type.as_str();

            match handle {
                "corner-tl" => {
                    let fixed_x = state.original_x + state.original_width;
                    let fixed_y = state.original_y + state.original_height;
                    ann.x = mouse_x.min(fixed_x);
                    ann.y = mouse_y.min(fixed_y);
                    ann.width = (fixed_x - ann.x).abs();
                    ann.height = (fixed_y - ann.y).abs();
                }
                "corner-tr" => {
                    let fixed_x = state.original_x;
                    let fixed_y = state.original_y + state.original_height;
                    ann.x = mouse_x.min(fixed_x);
                    ann.y = mouse_y.min(fixed_y);
                    ann.width = (mouse_x - fixed_x).abs();
                    ann.height = (fixed_y - ann.y).abs();
                }
                "corner-bl" => {
                    let fixed_x = state.original_x + state.original_width;
                    let fixed_y = state.original_y;
                    ann.x = mouse_x.min(fixed_x);
                    ann.y = mouse_y.min(fixed_y);
                    ann.width = (fixed_x - ann.x).abs();
                    ann.height = (mouse_y - fixed_y).abs();
                }
                "corner-br" => {
                    let fixed_x = state.original_x;
                    let fixed_y = state.original_y;
                    ann.x = mouse_x.min(fixed_x);
                    ann.y = mouse_y.min(fixed_y);
                    ann.width = (mouse_x - fixed_x).abs();
                    ann.height = (mouse_y - fixed_y).abs();
                }
                "edge-t" => {
                    let fixed_y = state.original_y + state.original_height;
                    ann.y = mouse_y.min(fixed_y);
                    ann.height = (fixed_y - ann.y).abs();
                }
                "edge-r" => {
                    let fixed_x = state.original_x;
                    ann.width = (mouse_x - fixed_x).max(1.0);
                }
                "edge-b" => {
                    let fixed_y = state.original_y;
                    ann.height = (mouse_y - fixed_y).max(1.0);
                }
                "edge-l" => {
                    let fixed_x = state.original_x + state.original_width;
                    ann.x = mouse_x.min(fixed_x);
                    ann.width = (fixed_x - ann.x).abs();
                }
                _ => {}
            }

            if ann.state == "Pending" {
                ann.state = "Accepted".into();
            }
            annotations.set_row_data(index, ann);
        }
    });
}

fn setup_finish_resize(ui: &AppWindow) {
    let ui_weak = ui.as_weak();
    ui.on_finish_resize(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_text("Resize complete".into());
        }
        println!("Resize finished");
    });
}
