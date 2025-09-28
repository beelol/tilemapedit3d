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
            let floor_selected = !state.rotate_mode && state.current_kind == TileKind::Floor;
            if ui.selectable_label(floor_selected, "Floor").clicked() {
                state.rotate_mode = false;
                state.current_kind = TileKind::Floor;
            }
            let ramp_selected = !state.rotate_mode && state.current_kind == TileKind::Ramp;
            if ui.selectable_label(ramp_selected, "Ramp").clicked() {
                state.rotate_mode = false;
                state.current_kind = TileKind::Ramp;
            }
            if ui
                .selectable_label(state.rotate_mode, "Rotate Ramp")
                .clicked()
            {
                state.rotate_mode = true;
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
                if let Ok(mut m) = load_map("map.json") {
                    crate::editor::auto_orient_entire_map(&mut m);
                    state.map = m;
                    state.map_dirty = true;
                }
            }
        });
    });
}
