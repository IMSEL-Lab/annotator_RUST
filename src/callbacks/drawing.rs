//! Drawing callbacks for bbox/point creation.
//!
//! Handles: start_drawing, update_drawing, finish_drawing, cancel_drawing

use crate::state::{snapshot_annotations, DrawState, UndoHistory};
use crate::{Annotation, AppWindow};
use slint::{ComponentHandle, Model};
use std::cell::RefCell;
use std::rc::Rc;

/// Sets up all drawing-related callbacks on the UI.
pub fn setup_drawing_callbacks(
    ui: &AppWindow,
    draw_state: Rc<RefCell<DrawState>>,
    annotations: Rc<slint::VecModel<Annotation>>,
    undo_history: Rc<RefCell<UndoHistory>>,
) {
    setup_start_drawing(ui, draw_state.clone(), annotations.clone());
    setup_update_drawing(ui, draw_state.clone());
    setup_finish_drawing(ui, draw_state, annotations, undo_history);
    setup_cancel_drawing(ui);
}

fn setup_start_drawing(
    ui: &AppWindow,
    draw_state: Rc<RefCell<DrawState>>,
    annotations: Rc<slint::VecModel<Annotation>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_start_drawing(move |x, y| {
        let mut state = draw_state.borrow_mut();
        state.start_x = x;
        state.start_y = y;

        // Deselect all annotations when starting a new one
        for i in 0..annotations.row_count() {
            if let Some(mut ann) = annotations.row_data(i) {
                if ann.selected {
                    ann.selected = false;
                    annotations.set_row_data(i, ann);
                }
            }
        }

        if let Some(ui) = ui_weak.upgrade() {
            ui.set_show_preview(true);
            ui.set_preview_x(x);
            ui.set_preview_y(y);
            ui.set_preview_width(0.0);
            ui.set_preview_height(0.0);
        }
    });
}

fn setup_update_drawing(ui: &AppWindow, draw_state: Rc<RefCell<DrawState>>) {
    let ui_weak = ui.as_weak();
    ui.on_update_drawing(move |x, y| {
        let state = draw_state.borrow();

        if let Some(ui) = ui_weak.upgrade() {
            let min_x = state.start_x.min(x);
            let min_y = state.start_y.min(y);
            let width = (x - state.start_x).abs();
            let height = (y - state.start_y).abs();

            ui.set_preview_x(min_x);
            ui.set_preview_y(min_y);
            ui.set_preview_width(width);
            ui.set_preview_height(height);
        }
    });
}

fn setup_finish_drawing(
    ui: &AppWindow,
    draw_state: Rc<RefCell<DrawState>>,
    annotations: Rc<slint::VecModel<Annotation>>,
    undo_history: Rc<RefCell<UndoHistory>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_finish_drawing(move |x, y| {
        // Push current state to undo history before creating new annotation
        undo_history.borrow_mut().push(snapshot_annotations(&annotations));

        let mut state = draw_state.borrow_mut();

        if let Some(ui) = ui_weak.upgrade() {
            ui.set_show_preview(false);

            let min_x = state.start_x.min(x);
            let min_y = state.start_y.min(y);
            let width = (x - state.start_x).abs();
            let height = (y - state.start_y).abs();

            let tool = ui.get_current_tool();
            let class = ui.get_current_class();

            if tool.as_str().starts_with("BBox") {
                // Create bbox annotation only if size is reasonable (at least 5 pixels)
                if width >= 5.0 && height >= 5.0 {
                    annotations.push(Annotation {
                        id: state.next_id,
                        r#type: "bbox".into(),
                        x: min_x,
                        y: min_y,
                        width,
                        height,
                        rotation: 0.0,
                        selected: false,
                        class,
                        state: "Manual".into(),
                        vertices: "".into(),
                        polygon_vertices: Default::default(),
                        polygon_path_commands: "".into(),
                    });
                    state.next_id += 1;
                }
            } else if tool.as_str().starts_with("Point") {
                // Create point annotation at click location (no minimum size)
                annotations.push(Annotation {
                    id: state.next_id,
                    r#type: "point".into(),
                    x,
                    y,
                    width: 0.0,
                    height: 0.0,
                    rotation: 0.0,
                    selected: false,
                    class,
                    state: "Manual".into(),
                    vertices: "".into(),
                    polygon_vertices: Default::default(),
                    polygon_path_commands: "".into(),
                });
                state.next_id += 1;
            }
        }
    });
}

fn setup_cancel_drawing(ui: &AppWindow) {
    let ui_weak = ui.as_weak();
    ui.on_cancel_drawing(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_show_preview(false);
        }
    });
}
