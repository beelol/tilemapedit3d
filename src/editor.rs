use crate::terrain;
use crate::types::*;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy_egui::EguiContexts;

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorState>()
            .add_systems(Startup, spawn_editor_assets)
            .add_systems(
                Update,
                (
                    update_hover,
                    paint_tiles,
                    rotate_ramps,
                    rebuild_terrain_mesh,
                    draw_hover_highlight,
                ),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EditorTool {
    Paint,
    RotateRamp,
}

#[derive(Resource)]
pub struct EditorState {
    pub current_tool: EditorTool,
    pub current_kind: TileKind,
    pub current_elev: i8, // -1..3
    pub hover: Option<(u32, u32)>,
    pub map: TileMap,
    pub map_dirty: bool,
}
impl Default for EditorState {
    fn default() -> Self {
        Self {
            current_tool: EditorTool::Paint,
            current_kind: TileKind::Floor,
            current_elev: 0,
            hover: None,
            map: TileMap::new(64, 64),
            map_dirty: true,
        }
    }
}

#[derive(Resource)]
struct TerrainVisual {
    mesh: Handle<Mesh>,
}

fn spawn_editor_assets(
    mut commands: Commands,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let terrain_material = mats.add(StandardMaterial {
        base_color: Color::rgb(0.35, 0.55, 0.2),
        perceptual_roughness: 0.8,
        metallic: 0.0,
        ..default()
    });

    let terrain_mesh = meshes.add(Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    ));

    let terrain_material = mats.add(StandardMaterial {
        base_color_texture: Some(
            asset_server.load("textures/terrain/rocky_terrain_02_diff_1k.png"),
        ),
        normal_map_texture: Some(
            asset_server.load("textures/terrain/rocky_terrain_02_nor_gl_1k_fixed.exr"),
        ),
        metallic: 0.0,
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: terrain_mesh.clone(),
        material: terrain_material.clone(),
        transform: Transform::default(),
        ..default()
    });

    commands.insert_resource(TerrainVisual { mesh: terrain_mesh });
}

// Raycast to ground plane at chosen elevation (use current_elev for edit layer)
fn update_hover(
    mut state: ResMut<EditorState>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut egui: EguiContexts,
) {
    let (cam, cam_xform) = cameras.single();
    let win = windows.single();

    if egui.ctx_mut().wants_pointer_input() {
        state.hover = None;
        return;
    }

    let Some(cursor) = win.cursor_position() else {
        state.hover = None;
        return;
    };

    if let Some(ray) = cam.viewport_to_world(cam_xform, cursor) {
        // --- Step 1: flat projection (y = 0) candidate
        let guess_hit = ray.origin + ray.direction * ((0.0 - ray.origin.y) / ray.direction.y);
        let tx = (guess_hit.x / TILE_SIZE).floor() as i32;
        let ty = (guess_hit.z / TILE_SIZE).floor() as i32;

        if tx >= 0 && ty >= 0 && (tx as u32) < state.map.width && (ty as u32) < state.map.height {
            // Look up elevation at this flat tile
            let idx = (ty as u32 * state.map.width + tx as u32) as usize;
            let elev = state.map.tiles[idx].elevation as f32 * TILE_HEIGHT;

            // --- Step 2: recompute ray-plane hit at elevation
            let t = (elev - ray.origin.y) / ray.direction.y;
            if t.is_finite() && t > 0.0 {
                let hit = ray.origin + ray.direction * t;
                let x2 = (hit.x / TILE_SIZE).floor() as i32;
                let y2 = (hit.z / TILE_SIZE).floor() as i32;

                // Prefer elevated if it resolves to the same tile coords
                if x2 == tx && y2 == ty {
                    state.hover = Some((x2 as u32, y2 as u32));
                    return;
                }
            }

            // --- Fallback to flat tile
            state.hover = Some((tx as u32, ty as u32));
            return;
        }
    }

    // --- Nothing hit
    state.hover = None;
}

fn paint_tiles(
    buttons: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<EditorState>,
    mut egui: EguiContexts,
) {
    if egui.ctx_mut().wants_pointer_input() {
        return;
    }
    if state.current_tool != EditorTool::Paint {
        return;
    }
    if buttons.pressed(MouseButton::Left) {
        if let Some((x, y)) = state.hover {
            let kind = state.current_kind;
            let elevation = state.current_elev;
            let state_ref = &mut *state;
            let current = state_ref.map.get(x, y);
            let target_ramp_direction = if kind == TileKind::Ramp {
                let base = elevation as f32 * TILE_HEIGHT;
                let mut candidates = ramp_targets(&state_ref.map, x, y, base);
                if let Some(existing) = current.ramp_direction {
                    if candidates.contains(&existing) {
                        Some(existing)
                    } else {
                        candidates.first().copied()
                    }
                } else {
                    candidates.first().copied()
                }
            } else {
                None
            };
            if current.kind != kind
                || current.elevation != elevation
                || current.ramp_direction != target_ramp_direction
            {
                let tile_type = current.tile_type.clone();
                state_ref.map.set(
                    x,
                    y,
                    Tile {
                        kind,
                        elevation,
                        tile_type,
                        x,
                        y,
                        ramp_direction: target_ramp_direction,
                    },
                );
                state_ref.map_dirty = true;
            }
        }
    }
}

fn rotate_ramps(
    buttons: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<EditorState>,
    mut egui: EguiContexts,
) {
    if egui.ctx_mut().wants_pointer_input() {
        return;
    }
    if state.current_tool != EditorTool::RotateRamp {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some((x, y)) = state.hover else {
        return;
    };

    let base_tile = state.map.get(x, y).clone();
    if base_tile.kind != TileKind::Ramp {
        return;
    }

    let base_height = base_tile.elevation as f32 * TILE_HEIGHT;
    let candidates = ramp_targets(&state.map, x, y, base_height);
    if candidates.is_empty() {
        return;
    }

    let next_direction = match base_tile.ramp_direction {
        Some(current) => {
            if let Some(idx) = candidates.iter().position(|&dir| dir == current) {
                candidates[(idx + 1) % candidates.len()]
            } else {
                candidates[0]
            }
        }
        None => candidates[0],
    };

    if base_tile.ramp_direction == Some(next_direction) {
        return;
    }

    let mut updated = base_tile;
    updated.ramp_direction = Some(next_direction);
    state.map.set(x, y, updated);
    state.map_dirty = true;
}

fn ramp_targets(map: &TileMap, x: u32, y: u32, base: f32) -> Vec<RampDirection> {
    let mut results = Vec::new();
    for dir in RampDirection::ALL {
        let (dx, dy) = dir.offset();
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx < 0 || ny < 0 {
            continue;
        }
        let (ux, uy) = (nx as u32, ny as u32);
        if ux >= map.width || uy >= map.height {
            continue;
        }
        let neighbor = map.get(ux, uy);
        let height = neighbor.elevation as f32 * TILE_HEIGHT;
        if height < base {
            results.push(dir);
        }
    }
    results
}

fn rebuild_terrain_mesh(
    mut state: ResMut<EditorState>,
    mut meshes: ResMut<Assets<Mesh>>,
    visual: Res<TerrainVisual>,
) {
    if !state.map_dirty {
        return;
    }
    state.map_dirty = false;

    let mesh = terrain::build_map_mesh(&state.map);
    if let Some(existing) = meshes.get_mut(&visual.mesh) {
        *existing = mesh;
    }
}

fn draw_hover_highlight(mut gizmos: Gizmos, state: Res<EditorState>) {
    if let Some((x, y)) = state.hover {
        let heights = terrain::tile_corner_heights(&state.map, x, y);
        let offset = 0.02;
        let x0 = x as f32 * TILE_SIZE;
        let x1 = x0 + TILE_SIZE;
        let z0 = y as f32 * TILE_SIZE;
        let z1 = z0 + TILE_SIZE;
        gizmos.linestrip(
            [
                Vec3::new(x0, heights[terrain::CORNER_NW] + offset, z0),
                Vec3::new(x1, heights[terrain::CORNER_NE] + offset, z0),
                Vec3::new(x1, heights[terrain::CORNER_SE] + offset, z1),
                Vec3::new(x0, heights[terrain::CORNER_SW] + offset, z1),
                Vec3::new(x0, heights[terrain::CORNER_NW] + offset, z0),
            ],
            Color::srgb(0.0, 1.0, 0.0),
        );
    }
}
