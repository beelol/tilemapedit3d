mod controls;
mod editor;
mod io;
mod types;
mod ui;
mod camera;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use camera::CameraPlugin;
use controls::ControlsPlugin;
use editor::EditorPlugin;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin))
        .add_plugins((CameraPlugin, ControlsPlugin, EditorPlugin, UiPlugin))
        .add_systems(Startup, setup_light)
        .run();
}

fn setup_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight { illuminance: 20_000.0, shadows_enabled: false, ..default() },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.2, -0.8, 0.0)),
        ..default()
    });
}
