//! Selection callbacks for annotation management.
//!
//! Handles: select, deselect_all, select_all, delete_selected

use crate::state::{snapshot_annotations, UndoHistory};
use crate::{Annotation, AppWindow};
use slint::{ComponentHandle, Model};
use std::cell::RefCell;
use std::rc::Rc;

/// Sets up all selection-related callbacks on the UI.
pub fn setup_selection_callbacks(
    ui: &AppWindow,
    annotations: Rc<slint::VecModel<Annotation>>,
    undo_history: Rc<RefCell<UndoHistory>>,
) {
    setup_select_annotation(ui, annotations.clone());
    setup_deselect_all(ui, annotations.clone());
    setup_select_all(ui, annotations.clone());
    setup_delete_selected(ui, annotations, undo_history);
}

fn setup_select_annotation(ui: &AppWindow, annotations: Rc<slint::VecModel<Annotation>>) {
    let ui_weak = ui.as_weak();
    // Multi-selection support: Ctrl toggles, Shift extends range, normal click selects only one
    ui.on_select_annotation(move |index| {
        let ui = ui_weak.upgrade().unwrap();
        let shift_held = ui.get_shift_key_held();
        let ctrl_held = ui.get_ctrl_key_held();
        let count = annotations.row_count();
        let target_index = index as usize;

        if ctrl_held {
            // Ctrl+Click: Toggle selection of clicked annotation
            if let Some(mut data) = annotations.row_data(target_index) {
                data.selected = !data.selected;
                annotations.set_row_data(target_index, data);
            }
        } else if shift_held {
            // Shift+Click: Extend selection from last selected to this one
            let mut last_selected: Option<usize> = None;
            for i in 0..count {
                if let Some(data) = annotations.row_data(i) {
                    if data.selected {
                        last_selected = Some(i);
                    }
                }
            }

            if let Some(start) = last_selected {
                let (range_start, range_end) = if start < target_index {
                    (start, target_index)
                } else {
                    (target_index, start)
                };

                for i in range_start..=range_end {
                    if let Some(mut data) = annotations.row_data(i) {
                        data.selected = true;
                        annotations.set_row_data(i, data);
                    }
                }
            } else {
                // No existing selection, just select this one
                for i in 0..count {
                    if let Some(mut data) = annotations.row_data(i) {
                        data.selected = i == target_index;
                        annotations.set_row_data(i, data);
                    }
                }
            }
        } else {
            // Normal click: Select only this annotation
            for i in 0..count {
                if let Some(mut data) = annotations.row_data(i) {
                    data.selected = i == target_index;
                    annotations.set_row_data(i, data);
                }
            }
        }
    });
}

fn setup_deselect_all(ui: &AppWindow, annotations: Rc<slint::VecModel<Annotation>>) {
    ui.on_deselect_all(move || {
        let count = annotations.row_count();
        for i in 0..count {
            if let Some(mut data) = annotations.row_data(i) {
                if data.selected {
                    data.selected = false;
                    annotations.set_row_data(i, data);
                }
            }
        }
    });
}

fn setup_select_all(ui: &AppWindow, annotations: Rc<slint::VecModel<Annotation>>) {
    ui.on_select_all(move || {
        let count = annotations.row_count();
        for i in 0..count {
            if let Some(mut data) = annotations.row_data(i) {
                data.selected = true;
                annotations.set_row_data(i, data);
            }
        }
    });
}

fn setup_delete_selected(
    ui: &AppWindow,
    annotations: Rc<slint::VecModel<Annotation>>,
    undo_history: Rc<RefCell<UndoHistory>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_delete_selected(move || {
        // Push current state to undo history before deletion
        undo_history.borrow_mut().push(snapshot_annotations(&annotations));

        let mut deleted_count = 0;
        let count = annotations.row_count();
        for i in 0..count {
            if let Some(mut ann) = annotations.row_data(i) {
                if ann.selected {
                    ann.state = "Rejected".into();
                    ann.selected = false;
                    annotations.set_row_data(i, ann);
                    deleted_count += 1;
                }
            }
        }

        if deleted_count > 0 {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_status_text(format!("Deleted {} annotation(s)", deleted_count).into());
            }
        }
    });
}
