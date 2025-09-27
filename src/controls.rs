use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_egui::EguiContexts;

pub struct ControlsPlugin;
impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_move);
    }
}

fn camera_move(
    mut q_cam: Query<&mut Transform, With<Camera>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut scroll: EventReader<MouseWheel>,
    mut egui: EguiContexts,
    time: Res<Time>,
) {
    if egui.ctx_mut().wants_pointer_input() || egui.ctx_mut().wants_keyboard_input() {
        return;
    }

    let mut t: Mut<Transform> = q_cam.single_mut();
    let f: f32 = 20.0 * time.delta_seconds();

    let right: Dir3 = t.right();
    let forward: Vec3 = (t.forward().xz().normalize_or_zero().extend(0.0));

    if keys.pressed(KeyCode::KeyW) { t.translation += forward * f; }
    if keys.pressed(KeyCode::KeyS) { t.translation -= forward * f; }
    if keys.pressed(KeyCode::KeyA) { t.translation -= right.as_vec3() * f; }
    if keys.pressed(KeyCode::KeyD) { t.translation += right.as_vec3() * f; }

    for ev in scroll.read() {
        let forward = t.forward().as_vec3();
        t.translation += forward * (ev.y * 0.5);
    }
}

