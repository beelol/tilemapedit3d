use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use crate::types::*;
use crate::io::{save_map, load_map};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) { app.add_systems(Update, ui_panel); }
}

fn ui_panel(
    mut egui_ctx: EguiContexts,
    mut state: ResMut<crate::editor::EditorState>,
){
    egui::TopBottomPanel::top("toolbar").show(egui_ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Tool:");
            ui.selectable_value(&mut state.current_kind, TileKind::Floor, "Floor");
            ui.selectable_value(&mut state.current_kind, TileKind::Ramp, "Ramp");

            ui.separator();
            ui.label("Elevation:");
            for e in 0..=3 { ui.selectable_value(&mut state.current_elev, e, format!("{e}")); }

            ui.separator();
            if ui.button("Save").clicked() { save_map("map.json", &state.map).ok(); }
            if ui.button("Load").clicked() { if let Ok(m)=load_map("map.json"){ state.map = m; } }
        });
    });
}
