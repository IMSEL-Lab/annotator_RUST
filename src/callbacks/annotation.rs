//! Annotation manipulation callbacks.
//!
//! Handles: delete, classify, undo, redo, copy, paste operations

use crate::state::{replace_annotations, snapshot_annotations, UndoHistory};
use crate::{Annotation, AppWindow};
use slint::{ComponentHandle, Model};
use std::cell::RefCell;
use std::rc::Rc;

/// Sets up all annotation manipulation callbacks on the UI.
pub fn setup_annotation_callbacks(
    ui: &AppWindow,
    annotations: Rc<slint::VecModel<Annotation>>,
    undo_history: Rc<RefCell<UndoHistory>>,
    clipboard: Rc<RefCell<Vec<Annotation>>>,
) {
    setup_delete_annotation_at(ui, annotations.clone(), undo_history.clone());
    setup_delete_annotation(ui, annotations.clone(), undo_history.clone());
    setup_classify_at(ui, annotations.clone(), undo_history.clone());
    setup_classify_selected(ui, annotations.clone(), undo_history.clone());
    setup_undo_action(ui, annotations.clone(), undo_history.clone());
    setup_redo_action(ui, annotations.clone(), undo_history.clone());
    setup_copy_annotation(ui, annotations.clone(), clipboard.clone());
    setup_paste_annotation(ui, annotations, undo_history, clipboard);
}

fn setup_delete_annotation_at(
    ui: &AppWindow,
    annotations: Rc<slint::VecModel<Annotation>>,
    undo_history: Rc<RefCell<UndoHistory>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_delete_annotation_at(move |x, y| {
        undo_history.borrow_mut().push(snapshot_annotations(&annotations));

        let count = annotations.row_count();
        for i in (0..count).rev() {
            if let Some(ann) = annotations.row_data(i) {
                if ann.state == "Rejected" {
                    continue;
                }
                let inside = if ann.r#type.as_str() == "point" {
                    let dx = x - ann.x;
                    let dy = y - ann.y;
                    (dx * dx + dy * dy).sqrt() < 10.0
                } else {
                    x >= ann.x && x <= ann.x + ann.width && y >= ann.y && y <= ann.y + ann.height
                };

                if inside {
                    let mut rejected = ann;
                    rejected.state = "Rejected".into();
                    rejected.selected = false;
                    annotations.set_row_data(i, rejected);
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_text("Annotation deleted".into());
                    }
                    break;
                }
            }
        }
    });
}

fn setup_delete_annotation(
    ui: &AppWindow,
    annotations: Rc<slint::VecModel<Annotation>>,
    undo_history: Rc<RefCell<UndoHistory>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_delete_annotation(move |index| {
        undo_history.borrow_mut().push(snapshot_annotations(&annotations));

        if let Some(mut ann) = annotations.row_data(index as usize) {
            ann.state = "Rejected".into();
            ann.selected = false;
            annotations.set_row_data(index as usize, ann);
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_status_text("Annotation deleted (double-click)".into());
            }
        }
    });
}

fn setup_classify_at(
    ui: &AppWindow,
    annotations: Rc<slint::VecModel<Annotation>>,
    undo_history: Rc<RefCell<UndoHistory>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_classify_at(move |x, y, new_class| {
        undo_history.borrow_mut().push(snapshot_annotations(&annotations));

        let count = annotations.row_count();
        for i in (0..count).rev() {
            if let Some(mut ann) = annotations.row_data(i) {
                if ann.state == "Rejected" {
                    continue;
                }
                let inside = if ann.r#type.as_str() == "point" {
                    let dx = x - ann.x;
                    let dy = y - ann.y;
                    (dx * dx + dy * dy).sqrt() < 10.0
                } else {
                    x >= ann.x && x <= ann.x + ann.width && y >= ann.y && y <= ann.y + ann.height
                };

                if inside {
                    ann.class = new_class;
                    if ann.state == "Pending" {
                        ann.state = "Accepted".into();
                    }
                    annotations.set_row_data(i, ann);
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_text(format!("Annotation reclassified to {}", new_class).into());
                    }
                    break;
                }
            }
        }
    });
}

fn setup_classify_selected(
    ui: &AppWindow,
    annotations: Rc<slint::VecModel<Annotation>>,
    undo_history: Rc<RefCell<UndoHistory>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_classify_selected(move |new_class| {
        undo_history.borrow_mut().push(snapshot_annotations(&annotations));

        let mut updated = false;
        let count = annotations.row_count();
        for i in 0..count {
            if let Some(mut ann) = annotations.row_data(i) {
                if ann.selected && ann.state != "Rejected" {
                    ann.class = new_class;
                    if ann.state == "Pending" {
                        ann.state = "Accepted".into();
                    }
                    annotations.set_row_data(i, ann);
                    updated = true;
                }
            }
        }

        if updated {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_status_text(format!("Selected annotation set to class {}", new_class).into());
            }
        }
    });
}

fn setup_undo_action(
    ui: &AppWindow,
    annotations: Rc<slint::VecModel<Annotation>>,
    undo_history: Rc<RefCell<UndoHistory>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_undo_action(move || {
        let current = snapshot_annotations(&annotations);
        if let Some(previous) = undo_history.borrow_mut().undo(current) {
            replace_annotations(&annotations, previous);
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_status_text("Undo".into());
            }
        } else if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_text("Nothing to undo".into());
        }
    });
}

fn setup_redo_action(
    ui: &AppWindow,
    annotations: Rc<slint::VecModel<Annotation>>,
    undo_history: Rc<RefCell<UndoHistory>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_redo_action(move || {
        let current = snapshot_annotations(&annotations);
        if let Some(next) = undo_history.borrow_mut().redo(current) {
            replace_annotations(&annotations, next);
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_status_text("Redo".into());
            }
        } else if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_text("Nothing to redo".into());
        }
    });
}

fn setup_copy_annotation(
    ui: &AppWindow,
    annotations: Rc<slint::VecModel<Annotation>>,
    clipboard: Rc<RefCell<Vec<Annotation>>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_copy_annotation(move || {
        let count = annotations.row_count();
        let mut copied_annotations = Vec::new();

        for i in 0..count {
            if let Some(ann) = annotations.row_data(i) {
                if ann.selected {
                    copied_annotations.push(ann.clone());
                }
            }
        }

        if copied_annotations.is_empty() {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_status_text("No annotation selected to copy".into());
            }
        } else {
            *clipboard.borrow_mut() = copied_annotations.clone();
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_status_text(format!("Copied {} annotation(s)", copied_annotations.len()).into());
            }
        }
    });
}

fn setup_paste_annotation(
    ui: &AppWindow,
    annotations: Rc<slint::VecModel<Annotation>>,
    undo_history: Rc<RefCell<UndoHistory>>,
    clipboard: Rc<RefCell<Vec<Annotation>>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_paste_annotation(move || {
        let copied_anns = clipboard.borrow().clone();

        if copied_anns.is_empty() {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_status_text("No annotation to paste".into());
            }
            return;
        }

        let snapshot = snapshot_annotations(&annotations);
        undo_history.borrow_mut().push(snapshot);

        let existing: Vec<_> = (0..annotations.row_count())
            .filter_map(|i| annotations.row_data(i))
            .collect();
        let mut next_id = crate::state::next_id_from_annotations(&existing, 1);

        let offset_x = 0.05;
        let offset_y = 0.05;

        for copied_ann in copied_anns.iter() {
            let mut new_ann = copied_ann.clone();
            new_ann.id = next_id;
            new_ann.x += offset_x;
            new_ann.y += offset_y;
            new_ann.selected = false;

            annotations.push(new_ann);
            next_id += 1;
        }

        if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_text(format!("Pasted {} annotation(s)", copied_anns.len()).into());
        }
    });
}
