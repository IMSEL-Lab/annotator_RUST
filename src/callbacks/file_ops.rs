//! File operation callbacks.
//!
//! Handles: save, open, new dataset, export COCO/VOC, and auto-save timer

use crate::state::{
    create_dataset_from_folder, load_dataset, load_image_from_entry, save_all, save_current_state,
    DatasetState, ViewState,
};
use crate::{classes, export, Annotation, AppWindow};
use slint::ComponentHandle;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

/// Type alias for the image loader closure
pub type ImageLoader = Rc<dyn Fn(usize)>;

/// Sets up all file operation callbacks on the UI.
pub fn setup_file_callbacks(
    ui: &AppWindow,
    loader: ImageLoader,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    annotations: Rc<slint::VecModel<Annotation>>,
    image_dimensions: Rc<RefCell<(f32, f32)>>,
    classes: Rc<RefCell<classes::ClassConfig>>,
) {
    setup_save_dataset(ui, dataset_state.clone(), annotations.clone(), image_dimensions.clone());
    setup_toggle_frame_completion(ui, dataset_state.clone());
    setup_open_dataset(ui, loader.clone(), dataset_state.clone(), classes.clone());
    setup_new_dataset(ui, loader, dataset_state.clone(), classes.clone());
    setup_export_coco(ui, dataset_state.clone(), classes.clone());
    setup_export_voc(ui, dataset_state.clone(), classes);
    setup_view_changed(ui, dataset_state.clone(), image_dimensions.clone());
    setup_auto_save_timer(ui, dataset_state, annotations, image_dimensions);
}

fn setup_save_dataset(
    ui: &AppWindow,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    annotations: Rc<slint::VecModel<Annotation>>,
    image_dimensions: Rc<RefCell<(f32, f32)>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_save_dataset(move || {
        if let (Ok(mut ds_opt), Some(ui)) = (dataset_state.try_borrow_mut(), ui_weak.upgrade()) {
            if let Some(ds) = ds_opt.as_mut() {
                save_current_state(ds, &annotations, &ui, *image_dimensions.borrow());
                match save_all(ds) {
                    Ok(_) => ui.set_status_text("Save successful".into()),
                    Err(e) => ui.set_status_text(format!("Save failed: {e}").into()),
                }
            }
        }
    });
}

fn setup_toggle_frame_completion(ui: &AppWindow, dataset_state: Rc<RefCell<Option<DatasetState>>>) {
    let ui_weak = ui.as_weak();
    ui.on_toggle_frame_completion(move || {
        if let (Ok(mut ds_opt), Some(ui)) = (dataset_state.try_borrow_mut(), ui_weak.upgrade()) {
            if let Some(ds) = ds_opt.as_mut() {
                let idx = ds.current_index;
                if idx < ds.completed_frames.len() {
                    ds.completed_frames[idx] = !ds.completed_frames[idx];
                    ui.set_frame_completed(ds.completed_frames[idx]);
                    let status = if ds.completed_frames[idx] {
                        "✓ Frame marked as complete"
                    } else {
                        "Frame marked as incomplete"
                    };
                    ui.set_status_text(status.into());
                }
            }
        }
    });
}

fn setup_open_dataset(
    ui: &AppWindow,
    loader: ImageLoader,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    classes: Rc<RefCell<classes::ClassConfig>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_open_dataset(move || {
        let file = rfd::FileDialog::new()
            .add_filter("Dataset JSON", &["json"])
            .set_title("Open Dataset")
            .pick_file();

        if let Some(path) = file {
            match load_dataset(&path) {
                Ok(state) => {
                    let len = state.entries.len();
                    let mut state = state;
                    state.stored_annotations = vec![None; len];
                    state.view_states = vec![None; len];
                    state.completed_frames = vec![false; len];

                    // Use class configuration from dataset if available
                    if let (Some(dataset_classes), Some(ui)) = (&state.class_config, ui_weak.upgrade()) {
                        *classes.borrow_mut() = dataset_classes.clone();

                        // Update class items in UI
                        use crate::utils::parse_color;
                        use crate::ClassItem;
                        let class_items: Vec<ClassItem> = dataset_classes
                            .classes
                            .iter()
                            .map(|c| ClassItem {
                                id: c.id,
                                name: c.name.clone().into(),
                                color: c
                                    .color
                                    .as_ref()
                                    .and_then(|hex| parse_color(hex))
                                    .unwrap_or(slint::Color::from_rgb_u8(128, 128, 128))
                                    .into(),
                                shortcut: c.shortcut.clone().unwrap_or_default().into(),
                            })
                            .collect();
                        ui.set_class_items(slint::ModelRc::new(slint::VecModel::from(class_items)));
                    }

                    *dataset_state.borrow_mut() = Some(state);

                    loader(0);

                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_text(format!("Loaded dataset: {}", path.display()).into());
                    }
                }
                Err(e) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_text(format!("Failed to load dataset: {e}").into());
                    }
                }
            }
        }
    });
}

fn setup_new_dataset(
    ui: &AppWindow,
    loader: ImageLoader,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    classes: Rc<RefCell<classes::ClassConfig>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_new_dataset(move || {
        let folder = rfd::FileDialog::new()
            .set_title("Select Folder with Images")
            .pick_folder();

        if let Some(folder_path) = folder {
            // Check if manifest.json already exists in the folder
            let manifest_path = folder_path.join("manifest.json");

            if manifest_path.exists() {
                // Load existing manifest
                match load_dataset(&manifest_path) {
                    Ok(state) => {
                        let len = state.entries.len();
                        let mut state = state;
                        state.stored_annotations = vec![None; len];
                        state.view_states = vec![None; len];
                        state.completed_frames = vec![false; len];

                        // Use class configuration from dataset if available
                        if let (Some(dataset_classes), Some(ui)) = (&state.class_config, ui_weak.upgrade()) {
                            *classes.borrow_mut() = dataset_classes.clone();

                            // Update class items in UI
                            use crate::utils::parse_color;
                            use crate::ClassItem;
                            let class_items: Vec<ClassItem> = dataset_classes
                                .classes
                                .iter()
                                .map(|c| ClassItem {
                                    id: c.id,
                                    name: c.name.clone().into(),
                                    color: c
                                        .color
                                        .as_ref()
                                        .and_then(|hex| parse_color(hex))
                                        .unwrap_or(slint::Color::from_rgb_u8(128, 128, 128))
                                        .into(),
                                    shortcut: c.shortcut.clone().unwrap_or_default().into(),
                                })
                                .collect();
                            ui.set_class_items(slint::ModelRc::new(slint::VecModel::from(class_items)));
                        }

                        *dataset_state.borrow_mut() = Some(state);

                        loader(0);

                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_status_text(
                                format!("Loaded existing dataset with {} images", len).into(),
                            );
                        }
                    }
                    Err(e) => {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_status_text(format!("Failed to load existing dataset: {e}").into());
                        }
                    }
                }
            } else {
                // Create new manifest with current class configuration
                match create_dataset_from_folder(&folder_path, Some(&classes.borrow())) {
                    Ok(manifest_path) => match load_dataset(&manifest_path) {
                        Ok(state) => {
                            let len = state.entries.len();
                            let mut state = state;
                            state.stored_annotations = vec![None; len];
                            state.view_states = vec![None; len];
                            state.completed_frames = vec![false; len];
                            *dataset_state.borrow_mut() = Some(state);

                            loader(0);

                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_status_text(
                                    format!("Created new dataset with {} images", len).into(),
                                );
                            }
                        }
                        Err(e) => {
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_status_text(format!("Failed to load new dataset: {e}").into());
                            }
                        }
                    },
                    Err(e) => {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_status_text(format!("Failed to create dataset: {e}").into());
                        }
                    }
                }
            }
        }
    });
}

fn setup_export_coco(
    ui: &AppWindow,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    classes: Rc<RefCell<classes::ClassConfig>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_export_coco(move || {
        let folder = rfd::FileDialog::new()
            .set_title("Select Export Folder")
            .pick_folder();

        if let Some(export_folder) = folder {
            if let Ok(ds_opt) = dataset_state.try_borrow() {
                if let Some(ds) = ds_opt.as_ref() {
                    let mut coco = export::coco::CocoDataset::new();

                    for class_def in &classes.borrow().classes {
                        coco.add_category(class_def.id, class_def.name.clone());
                    }

                    let mut ann_id = 1;

                    for (img_idx, entry) in ds.entries.iter().enumerate() {
                        let filename = entry
                            .image_path
                            .file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or("unknown.png")
                            .to_string();

                        let (width, height) = if let Ok(img) = load_image_from_entry(entry) {
                            let size = img.size();
                            (size.width as i32, size.height as i32)
                        } else {
                            (640, 480)
                        };

                        coco.images.push(export::coco::CocoImage {
                            id: (img_idx + 1) as i32,
                            width,
                            height,
                            file_name: filename,
                        });

                        if let Some(Some(annotations)) = ds.stored_annotations.get(img_idx) {
                            for ann in annotations {
                                let (bbox_opt, segmentation_opt, area_opt) =
                                    match ann.r#type.as_str() {
                                        "bbox" | "rbbox" => {
                                            let bbox = [
                                                ann.x as f64,
                                                ann.y as f64,
                                                ann.width as f64,
                                                ann.height as f64,
                                            ];
                                            let area = ann.width as f64 * ann.height as f64;
                                            (Some(bbox), None, Some(area))
                                        }
                                        "point" => {
                                            let bbox = [ann.x as f64, ann.y as f64, 1.0, 1.0];
                                            (Some(bbox), None, Some(1.0))
                                        }
                                        "polygon" => {
                                            let verts: Vec<f64> = ann
                                                .vertices
                                                .as_str()
                                                .split(',')
                                                .filter_map(|s| s.trim().parse().ok())
                                                .collect();
                                            let area = if verts.len() >= 6 {
                                                let mut a = 0.0;
                                                for i in 0..verts.len() / 2 {
                                                    let j = (i + 1) % (verts.len() / 2);
                                                    a += verts[i * 2] * verts[j * 2 + 1];
                                                    a -= verts[j * 2] * verts[i * 2 + 1];
                                                }
                                                (a / 2.0).abs()
                                            } else {
                                                0.0
                                            };
                                            (None, Some(vec![verts]), Some(area))
                                        }
                                        _ => continue,
                                    };

                                coco.annotations.push(export::coco::CocoAnnotation {
                                    id: ann_id,
                                    image_id: (img_idx + 1) as i32,
                                    category_id: ann.class,
                                    bbox: bbox_opt,
                                    segmentation: segmentation_opt,
                                    area: area_opt,
                                    iscrowd: 0,
                                });
                                ann_id += 1;
                            }
                        }
                    }

                    let coco_path = export_folder.join("annotations.json");
                    match coco.save(&coco_path) {
                        Ok(_) => {
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_status_text(
                                    format!(
                                        "Exported {} images with {} annotations to COCO JSON",
                                        coco.images.len(),
                                        coco.annotations.len()
                                    )
                                    .into(),
                                );
                            }
                        }
                        Err(e) => {
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_status_text(format!("Export failed: {e}").into());
                            }
                        }
                    }
                }
            }
        }
    });
}

fn setup_export_voc(
    ui: &AppWindow,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    classes: Rc<RefCell<classes::ClassConfig>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_export_voc(move || {
        let folder = rfd::FileDialog::new()
            .set_title("Select Export Folder")
            .pick_folder();

        if let Some(export_folder) = folder {
            if let Ok(ds_opt) = dataset_state.try_borrow() {
                if let Some(ds) = ds_opt.as_ref() {
                    let mut total_files = 0;
                    let mut total_annotations = 0;

                    for (img_idx, entry) in ds.entries.iter().enumerate() {
                        let filename = entry
                            .image_path
                            .file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or("unknown.png")
                            .to_string();

                        let (width, height) = if let Ok(img) = load_image_from_entry(entry) {
                            let size = img.size();
                            (size.width as i32, size.height as i32)
                        } else {
                            (640, 480)
                        };

                        let mut voc_ann =
                            export::voc::VocAnnotation::new(filename.clone(), width, height);

                        let mut has_annotations = false;
                        if let Some(Some(annotations)) = ds.stored_annotations.get(img_idx) {
                            for ann in annotations {
                                if ann.r#type.as_str() == "bbox" || ann.r#type.as_str() == "rbbox" {
                                    let class_name =
                                        classes::get_class_name(&classes.borrow(), ann.class);
                                    let xmin = ann.x as i32;
                                    let ymin = ann.y as i32;
                                    let xmax = (ann.x + ann.width) as i32;
                                    let ymax = (ann.y + ann.height) as i32;
                                    voc_ann.add_object(class_name, xmin, ymin, xmax, ymax);
                                    has_annotations = true;
                                    total_annotations += 1;
                                }
                            }
                        }

                        if has_annotations || true {
                            let xml_filename = Path::new(&filename).with_extension("xml");
                            let xml_path = export_folder.join(xml_filename);
                            if let Err(e) = voc_ann.save(&xml_path) {
                                if let Some(ui) = ui_weak.upgrade() {
                                    ui.set_status_text(format!("Export failed: {e}").into());
                                }
                                return;
                            }
                            total_files += 1;
                        }
                    }

                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_text(
                            format!(
                                "Exported {} XML files with {} annotations to Pascal VOC",
                                total_files, total_annotations
                            )
                            .into(),
                        );
                    }
                }
            }
        }
    });
}

fn setup_view_changed(
    ui: &AppWindow,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    image_dimensions: Rc<RefCell<(f32, f32)>>,
) {
    ui.on_view_changed(move |px, py, z| {
        if let Ok(mut ds_opt) = dataset_state.try_borrow_mut() {
            if let Some(ds) = ds_opt.as_mut() {
                ds.global_view = Some(ViewState {
                    pan_x: px,
                    pan_y: py,
                    zoom: z,
                });
                ds.last_view_image_size = Some(*image_dimensions.borrow());
            }
        }
    });
}

fn setup_auto_save_timer(
    ui: &AppWindow,
    dataset_state: Rc<RefCell<Option<DatasetState>>>,
    annotations: Rc<slint::VecModel<Annotation>>,
    image_dimensions: Rc<RefCell<(f32, f32)>>,
) {
    let ui_weak = ui.as_weak();
    slint::Timer::default().start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_secs(5),
        move || {
            if let (Ok(mut ds_opt), Some(ui)) = (dataset_state.try_borrow_mut(), ui_weak.upgrade())
            {
                if let Some(ds) = ds_opt.as_mut() {
                    save_current_state(ds, &annotations, &ui, *image_dimensions.borrow());
                    if let Err(e) = save_all(ds) {
                        ui.set_status_text(format!("Autosave failed: {e}").into());
                    }
                }
            }
        },
    );
}
