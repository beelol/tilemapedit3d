use crate::io::{load_map, save_map};
use crate::types::*;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_panel);
    }
}

fn ui_panel(mut egui_ctx: EguiContexts, mut state: ResMut<crate::editor::EditorState>) {
    egui::TopBottomPanel::top("toolbar").show(egui_ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Tool:");
            ui.selectable_value(&mut state.current_kind, TileKind::Floor, "Floor");
            ui.selectable_value(&mut state.current_kind, TileKind::Ramp, "Ramp");

            let can_rotate = state
                .hover
                .map(|(x, y)| state.map.get(x, y).kind == TileKind::Ramp)
                .unwrap_or(false);
            if ui
                .add_enabled(can_rotate, egui::Button::new("Rotate Ramp"))
                .clicked()
            {
                if let Some((x, y)) = state.hover {
                    let idx = state.map.idx(x, y);
                    let tile = &mut state.map.tiles[idx];
                    if tile.kind == TileKind::Ramp {
                        tile.ramp_orientation = match tile.ramp_orientation {
                            None => Some(RampDirection::North),
                            Some(RampDirection::North) => Some(RampDirection::East),
                            Some(RampDirection::East) => Some(RampDirection::South),
                            Some(RampDirection::South) => Some(RampDirection::West),
                            Some(RampDirection::West) => None,
                        };
                        state.map_dirty = true;
                    }
                }
            }

            ui.separator();
            ui.label("Elevation:");
            for e in 0..=3 {
                ui.selectable_value(&mut state.current_elev, e, format!("{e}"));
            }

            ui.separator();
            if ui.button("Save").clicked() {
                save_map("map.json", &state.map).ok();
            }
            if ui.button("Load").clicked() {
                if let Ok(m) = load_map("map.json") {
                    state.map = m;
                }
            }
        });
    });
}
