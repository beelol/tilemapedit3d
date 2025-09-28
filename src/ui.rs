use crate::editor::EditorTool;
use crate::io::{load_map, save_map};
use crate::types::*;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::texture::registry::TerrainTextureRegistry;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_panel);
    }
}

fn ui_panel(
    mut egui_ctx: EguiContexts,
    mut state: ResMut<crate::editor::EditorState>,
    textures: Res<TerrainTextureRegistry>,
) {
    let palette_items: Vec<_> = textures
        .iter()
        .map(|entry| PaletteItem {
            tile_type: entry.tile_type,
            name: entry.name.clone(),
            texture: egui_ctx.add_image(entry.icon.clone_weak()),
        })
        .collect();

    if palette_items
        .iter()
        .all(|item| item.tile_type != state.current_texture)
    {
        if let Some(first) = palette_items.first() {
            state.current_texture = first.tile_type;
        }
    }

    egui::TopBottomPanel::top("toolbar").show(egui_ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.selectable_value(&mut state.current_tool, EditorTool::Paint, "Paint");
            ui.selectable_value(
                &mut state.current_tool,
                EditorTool::RotateRamp,
                "Rotate Ramp",
            );

            if state.current_tool == EditorTool::Paint {
                ui.separator();
                ui.label("Tile:");
                ui.selectable_value(&mut state.current_kind, TileKind::Floor, "Floor");
                ui.selectable_value(&mut state.current_kind, TileKind::Ramp, "Ramp");

                ui.separator();
                ui.label("Elevation:");
                for e in 0..=3 {
                    ui.selectable_value(&mut state.current_elev, e, format!("{e}"));
                }
            }

            ui.separator();
            ui.label("Elevation:");
            for e in 0..=3 {
                ui.selectable_value(&mut state.current_elev, e, format!("{e}"));
            }

            ui.separator();
            let texture_slider = egui::Slider::new(&mut state.uv_scale, 0.5..=16.0)
                .logarithmic(true)
                .text("Texture Scale");
            if ui.add(texture_slider).changed() {
                state.map_dirty = true;
            }

            ui.separator();
            if ui.button("Save").clicked() {
                save_map("map.json", &state.map).ok();
            }
            if ui.button("Load").clicked() {
                if let Ok(m) = load_map("map.json") {
                    state.map = m;
                    state.map_dirty = true;
                }
            }
        });

        if !palette_items.is_empty() {
            ui.separator();
            ui.collapsing("Textures", |ui| {
                const COLUMNS: usize = 4;
                let mut grid = egui::Grid::new("texture_palette_grid")
                    .spacing([6.0, 6.0])
                    .num_columns(COLUMNS);

                grid.show(ui, |grid_ui| {
                    for (index, item) in palette_items.iter().enumerate() {
                        let is_selected = state.current_texture == item.tile_type;
                        let stroke = if is_selected {
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 122, 204))
                        } else {
                            egui::Stroke::NONE
                        };

                        let inner = egui::Frame::none()
                            .inner_margin(egui::Margin::same(2.0))
                            .stroke(stroke)
                            .show(grid_ui, |ui| {
                                ui.set_min_size(egui::Vec2::splat(36.0));
                                ui.centered_and_justified(|ui| {
                                    ui.add(egui::Image::new(egui::load::SizedTexture {
                                        id: item.texture,
                                        size: egui::vec2(32.0, 32.0),
                                    }));
                                });
                            });

                        let mut response = inner.response;

                        let response2 = response.on_hover_text(item.name.clone());

                        if response2.clicked() {
                            state.current_texture = item.tile_type;
                        }

                        if index % COLUMNS == COLUMNS - 1 {
                            grid_ui.end_row();
                        }
                    }

                    if palette_items.len() % COLUMNS != 0 {
                        grid_ui.end_row();
                    }
                });
            });
        }
    });
}

struct PaletteItem {
    tile_type: TileType,
    name: String,
    texture: egui::TextureId,
}
