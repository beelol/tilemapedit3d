mod camera;
mod controls;
mod editor;
mod grid_visual;
mod io;
mod terrain;
mod terrain_render;
mod texture;
mod types;
mod ui;

use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use camera::CameraPlugin;
use controls::ControlsPlugin;
use editor::EditorPlugin;
use terrain_render::TerrainMaterial;
use texture::TexturePlugin;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin))
        .add_plugins(MaterialPlugin::<TerrainMaterial>::default())
        .add_plugins((
            TexturePlugin,
            CameraPlugin,
            ControlsPlugin,
            EditorPlugin,
            UiPlugin,
        ))
        .add_systems(Startup, setup_light)
        .add_systems(Update, grid_visual::draw_grid)
        .run();
}

fn setup_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 20_000.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.2, -0.8, 0.0)),
        ..default()
    });
}
