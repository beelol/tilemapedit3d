use bevy::prelude::*;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) { app.add_systems(Startup, spawn_camera); }
}

// SC2 vibe: perspective, yaw ~45°, pitch ~60°, elevated and looking at origin.
fn spawn_camera(mut commands: Commands) {
    let yaw = 45f32.to_radians();
    let pitch = 60f32.to_radians();
    let dist = 30.0;

    let target = Vec3::ZERO;
    let dir = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0) * -Vec3::Z;
    let eye = target - dir * dist;

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(eye).looking_at(target, Vec3::Y),
            ..default()
        },
        Name::new("GameCamera"),
    ));
}
