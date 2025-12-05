//! Polygon annotation callbacks.
//!
//! Handles: add_vertex, finish, cancel polygon creation

use crate::state::{generate_path_commands, parse_vertices, DrawState};
use crate::{Annotation, AppWindow};
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;

/// Sets up all polygon-related callbacks on the UI.
pub fn setup_polygon_callbacks(
    ui: &AppWindow,
    draw_state: Rc<RefCell<DrawState>>,
    annotations: Rc<slint::VecModel<Annotation>>,
) {
    setup_add_polygon_vertex(ui, draw_state.clone());
    setup_finish_polygon(ui, draw_state.clone(), annotations);
    setup_cancel_polygon(ui, draw_state);
}

fn setup_add_polygon_vertex(ui: &AppWindow, draw_state: Rc<RefCell<DrawState>>) {
    let ui_weak = ui.as_weak();
    ui.on_add_polygon_vertex(move |x, y| {
        let mut state = draw_state.borrow_mut();
        state.polygon_vertices.push((x, y));

        if let Some(ui) = ui_weak.upgrade() {
            let vertices_str = state
                .polygon_vertices
                .iter()
                .map(|(vx, vy)| format!("{},{}", vx, vy))
                .collect::<Vec<_>>()
                .join(";");
            ui.set_polygon_preview_vertices(vertices_str.into());

            let preview_path = if state.polygon_vertices.len() >= 2 {
                let mut commands = format!(
                    "M {} {}",
                    state.polygon_vertices[0].0, state.polygon_vertices[0].1
                );
                for vertex in state.polygon_vertices.iter().skip(1) {
                    commands.push_str(&format!(" L {} {}", vertex.0, vertex.1));
                }
                commands
            } else if state.polygon_vertices.len() == 1 {
                format!(
                    "M {} {}",
                    state.polygon_vertices[0].0, state.polygon_vertices[0].1
                )
            } else {
                String::new()
            };
            ui.set_polygon_preview_path(preview_path.into());

            ui.set_status_text(
                format!(
                    "Polygon: {} vertices (hold S, release S or Tab/Enter to finish)",
                    state.polygon_vertices.len()
                )
                .into(),
            );
        }
        println!(
            "Vertex added at ({:.1}, {:.1}), total: {}",
            x,
            y,
            state.polygon_vertices.len()
        );
    });
}

fn setup_finish_polygon(
    ui: &AppWindow,
    draw_state: Rc<RefCell<DrawState>>,
    annotations: Rc<slint::VecModel<Annotation>>,
) {
    let ui_weak = ui.as_weak();
    ui.on_finish_polygon(move || {
        let mut state = draw_state.borrow_mut();

        if state.polygon_vertices.len() >= 3 {
            if let Some(ui) = ui_weak.upgrade() {
                let class = ui.get_current_class();

                let vertices_str = state
                    .polygon_vertices
                    .iter()
                    .map(|(x, y)| format!("{},{}", x, y))
                    .collect::<Vec<_>>()
                    .join(";");

                let xs: Vec<f32> = state.polygon_vertices.iter().map(|(x, _)| *x).collect();
                let ys: Vec<f32> = state.polygon_vertices.iter().map(|(_, y)| *y).collect();
                let min_x = xs.iter().cloned().fold(f32::INFINITY, f32::min);
                let max_x = xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                let min_y = ys.iter().cloned().fold(f32::INFINITY, f32::min);
                let max_y = ys.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

                let polygon_verts = parse_vertices(&vertices_str);
                let path_commands = generate_path_commands(&state.polygon_vertices);

                annotations.push(Annotation {
                    id: state.next_id,
                    r#type: "polygon".into(),
                    x: min_x,
                    y: min_y,
                    width: max_x - min_x,
                    height: max_y - min_y,
                    rotation: 0.0,
                    selected: false,
                    class,
                    state: "Manual".into(),
                    vertices: vertices_str.clone().into(),
                    polygon_vertices: std::rc::Rc::new(slint::VecModel::from(polygon_verts)).into(),
                    polygon_path_commands: path_commands.into(),
                });
                state.next_id += 1;
                println!(
                    "Polygon created with {} vertices: {}",
                    state.polygon_vertices.len(),
                    vertices_str
                );
                ui.set_status_text(
                    format!(
                        "Polygon created with {} vertices",
                        state.polygon_vertices.len()
                    )
                    .into(),
                );
            }
        }

        state.polygon_vertices.clear();
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_polygon_preview_vertices("".into());
            ui.set_polygon_preview_path("".into());
            ui.set_polygon_mode_active(false);
            ui.set_current_tool("Neutral".into());
        }
    });
}

fn setup_cancel_polygon(ui: &AppWindow, draw_state: Rc<RefCell<DrawState>>) {
    let ui_weak = ui.as_weak();
    ui.on_cancel_polygon(move || {
        let mut state = draw_state.borrow_mut();
        state.polygon_vertices.clear();

        if let Some(ui) = ui_weak.upgrade() {
            ui.set_polygon_preview_vertices("".into());
            ui.set_polygon_preview_path("".into());
            ui.set_polygon_mode_active(false);
            ui.set_current_tool("Neutral".into());
            ui.set_status_text("Polygon cancelled".into());
        }
    });
}
