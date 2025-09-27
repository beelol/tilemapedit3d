use bevy::prelude::*;

const GRID_SIZE: i32 = 20;     // how many cells across
const CELL_SIZE: f32 = 32.0;   // world units per cell

pub fn draw_grid(
    mut gizmos: Gizmos,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let half = GRID_SIZE as f32 * CELL_SIZE * 0.5;

    // --- Draw white grid lines ---
    for i in -GRID_SIZE..=GRID_SIZE {
        let x = i as f32 * CELL_SIZE;
        let z = i as f32 * CELL_SIZE;
        
        // Horizontal lines (along X axis)
        gizmos.line(Vec3::new(-half, 0.0, z), Vec3::new(half, 0.0, z), Color::WHITE);

        // Vertical lines (along Z axis)
        gizmos.line(Vec3::new(x, 0.0, -half), Vec3::new(x, 0.0, half), Color::WHITE);
    }

    // --- Hover highlight ---
    let window = windows.single();
    if let Some(cursor) = window.cursor_position() {
        let (camera, cam_transform) = camera_q.single();

        if let Some(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor) {
            // snap to grid cell
            let cell_x = (world_pos.x / CELL_SIZE).floor();
            let cell_y = (world_pos.y / CELL_SIZE).floor();

            let min = Vec2::new(cell_x * CELL_SIZE, cell_y * CELL_SIZE);
            let max = min + Vec2::splat(CELL_SIZE);

            // draw green outline
            gizmos.line_2d(min, Vec2::new(max.x, min.y), Color::linear_rgba(0., 100., 0., 0.)); // bottom
            gizmos.line_2d(min, Vec2::new(min.x, max.y), Color::linear_rgba(0., 100., 0., 0.)); // left
            gizmos.line_2d(max, Vec2::new(min.x, max.y), Color::linear_rgba(0., 100., 0., 0.)); // top
            gizmos.line_2d(max, Vec2::new(max.x, min.y), Color::linear_rgba(0., 100., 0., 0.)); // right
        }
    }
}
