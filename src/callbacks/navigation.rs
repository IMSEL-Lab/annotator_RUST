//! Navigation callbacks for image traversal.
//!
//! Handles: next, prev, first, last, and randomize image navigation.

use crate::state::{save_current_state, DatasetState};
use crate::{Annotation, AppWindow};
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;

/// Type alias for the image loader closure
pub type ImageLoader = Rc<dyn Fn(usize)>;

/// Sets up all navigation-related callbacks on the UI.
///
/// # Arguments
/// * `ui` - The AppWindow instance
/// * `loader` - Shared loader closure that loads image at given index
/// * `dataset_state` - Shared dataset state
/// * `annotations` - Shared annotations model
/// * `image_dimensions` - Current image dimensions
pub fn setup_navigation_callbacks(
    ui: &AppWindow,
    loader: ImageLoader,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    annotations: Rc<slint::VecModel<Annotation>>,
    image_dimensions: Rc<RefCell<(f32, f32)>>,
) {
    setup_next_image(ui, loader.clone(), dataset_state.clone(), annotations.clone(), image_dimensions.clone());
    setup_prev_image(ui, loader.clone(), dataset_state.clone(), annotations.clone(), image_dimensions.clone());
    setup_first_image(ui, loader.clone(), dataset_state.clone(), annotations.clone(), image_dimensions.clone());
    setup_last_image(ui, loader.clone(), dataset_state.clone(), annotations.clone(), image_dimensions.clone());
    setup_randomize(ui, loader, dataset_state, annotations, image_dimensions);
}

fn setup_next_image(
    ui: &AppWindow,
    loader: ImageLoader,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    annotations: Rc<slint::VecModel<Annotation>>,
    image_dimensions: Rc<RefCell<(f32, f32)>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_next_image(move || {
        let next_idx = {
            let mut ds_ref = dataset_state.borrow_mut();
            let Some(ds) = ds_ref.as_mut() else { return };
            if ds.entries.is_empty() {
                return;
            }

            if let Some(ui) = ui_weak.upgrade() {
                save_current_state(ds, &annotations, &ui, *image_dimensions.borrow());
            }

            let mut idx = ds.current_index;
            if idx + 1 < ds.entries.len() {
                idx += 1;
            }
            idx
        };

        loader(next_idx);
    });
}

fn setup_prev_image(
    ui: &AppWindow,
    loader: ImageLoader,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    annotations: Rc<slint::VecModel<Annotation>>,
    image_dimensions: Rc<RefCell<(f32, f32)>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_prev_image(move || {
        let prev_idx = {
            let mut ds_ref = dataset_state.borrow_mut();
            let Some(ds) = ds_ref.as_mut() else { return };
            if ds.entries.is_empty() {
                return;
            }

            if let Some(ui) = ui_weak.upgrade() {
                save_current_state(ds, &annotations, &ui, *image_dimensions.borrow());
            }

            if ds.current_index == 0 {
                0
            } else {
                ds.current_index - 1
            }
        };

        loader(prev_idx);
    });
}

fn setup_first_image(
    ui: &AppWindow,
    loader: ImageLoader,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    annotations: Rc<slint::VecModel<Annotation>>,
    image_dimensions: Rc<RefCell<(f32, f32)>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_first_image(move || {
        let first_idx = {
            let mut ds_ref = dataset_state.borrow_mut();
            let Some(ds) = ds_ref.as_mut() else { return };
            if ds.entries.is_empty() {
                return;
            }

            if let Some(ui) = ui_weak.upgrade() {
                save_current_state(ds, &annotations, &ui, *image_dimensions.borrow());
            }

            0
        };

        loader(first_idx);
    });
}

fn setup_last_image(
    ui: &AppWindow,
    loader: ImageLoader,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    annotations: Rc<slint::VecModel<Annotation>>,
    image_dimensions: Rc<RefCell<(f32, f32)>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_last_image(move || {
        let last_idx = {
            let mut ds_ref = dataset_state.borrow_mut();
            let Some(ds) = ds_ref.as_mut() else { return };
            if ds.entries.is_empty() {
                return;
            }

            if let Some(ui) = ui_weak.upgrade() {
                save_current_state(ds, &annotations, &ui, *image_dimensions.borrow());
            }

            ds.entries.len() - 1
        };

        loader(last_idx);
    });
}

fn setup_randomize(
    ui: &AppWindow,
    loader: ImageLoader,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    annotations: Rc<slint::VecModel<Annotation>>,
    image_dimensions: Rc<RefCell<(f32, f32)>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_randomize(move || {
        use rand::Rng;
        let random_idx = {
            let mut ds_ref = dataset_state.borrow_mut();
            let Some(ds) = ds_ref.as_mut() else { return };
            if ds.entries.is_empty() {
                return;
            }

            if let Some(ui) = ui_weak.upgrade() {
                save_current_state(ds, &annotations, &ui, *image_dimensions.borrow());
            }

            let mut rng = rand::thread_rng();
            rng.gen_range(0..ds.entries.len())
        };

        loader(random_idx);
    });
}
