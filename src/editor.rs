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
                    rebuild_terrain_mesh,
                    draw_hover_highlight,
                ),
            );
    }
}

#[derive(Resource)]
pub struct EditorState {
    pub current_kind: TileKind,
    pub current_elev: i8, // -1..3
    pub hover: Option<(u32, u32)>,
    pub map: TileMap,
    pub map_dirty: bool,
    pub rotate_mode: bool,
}
impl Default for EditorState {
    fn default() -> Self {
        Self {
            current_kind: TileKind::Floor,
            current_elev: 0,
            hover: None,
            map: TileMap::new(64, 64),
            map_dirty: true,
            rotate_mode: false,
        }
    }
}

// Simple green overlay material
#[derive(Resource)]
struct Materials {
    hover: Handle<StandardMaterial>,
}

#[derive(Resource)]
struct TerrainVisual {
    mesh: Handle<Mesh>,
}

fn spawn_editor_assets(
    mut commands: Commands,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let hover = mats.add(StandardMaterial {
        base_color: Color::rgba(0.0, 1.0, 0.0, 0.25),
        unlit: true,
        ..default()
    });
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

    commands.spawn(PbrBundle {
        mesh: terrain_mesh.clone(),
        material: terrain_material.clone(),
        transform: Transform::default(),
        ..default()
    });

    commands.insert_resource(Materials { hover });
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
        let plane_y = state.current_elev as f32 * TILE_HEIGHT;
        let t = (plane_y - ray.origin.y) / ray.direction.y;
        if t.is_finite() && t > 0.0 {
            let hit = ray.origin + ray.direction * t;
            let x = (hit.x / TILE_SIZE).floor() as i32;
            let y = (hit.z / TILE_SIZE).floor() as i32;
            if x >= 0 && y >= 0 && (x as u32) < state.map.width && (y as u32) < state.map.height {
                state.hover = Some((x as u32, y as u32));
                return;
            }
        }
    }
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

    if state.rotate_mode {
        if buttons.just_pressed(MouseButton::Left) {
            if let Some((x, y)) = state.hover {
                let idx = state.map.idx(x, y);
                let tile = state.map.tiles[idx].clone();
                if tile.kind == TileKind::Ramp {
                    let mut updated = tile.clone();
                    updated.orientation = tile.orientation.next();
                    updated.manual_orientation = true;
                    state.map.tiles[idx] = updated;
                    state.map_dirty = true;
                }
            }
        }
        return;
    }

    if buttons.pressed(MouseButton::Left) {
        if let Some((x, y)) = state.hover {
            let kind = state.current_kind;
            let elevation = state.current_elev;
            let state_ref = &mut *state;
            let idx = state_ref.map.idx(x, y);
            let mut current = state_ref.map.tiles[idx].clone();
            if current.kind != kind || current.elevation != elevation {
                current.kind = kind;
                current.elevation = elevation;
                current.x = x;
                current.y = y;
                match kind {
                    TileKind::Ramp => {
                        current.manual_orientation = false;
                    }
                    TileKind::Floor => {
                        current.manual_orientation = false;
                        current.orientation = Orientation4::North;
                    }
                }
                state_ref.map.tiles[idx] = current;
                state_ref.map_dirty = true;
                if auto_orient_around(&mut state_ref.map, x, y) {
                    state_ref.map_dirty = true;
                }
            }
        }
    }
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

// Draw a thin quad on hovered tile at its elevation
#[derive(Component)]
struct HoverMarker;

fn draw_hover_highlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mats: Res<Materials>,
    state: Res<EditorState>,
    existing: Query<Entity, With<HoverMarker>>,
) {
    // clear previous
    for e in &existing {
        commands.entity(e).despawn();
    }

    if let Some((x, y)) = state.hover {
        let min = Vec3::new(
            x as f32 * TILE_SIZE,
            state.current_elev as f32 * TILE_HEIGHT + 0.01,
            y as f32 * TILE_SIZE,
        );
        let size = Vec2::splat(TILE_SIZE);
        let mesh = Mesh::from(Rectangle::new(size.x, size.y));
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(mesh),
                material: mats.hover.clone(),
                transform: Transform::from_translation(
                    min + Vec3::new(TILE_SIZE * 0.5, 0.0, TILE_SIZE * 0.5),
                ) * Transform::from_rotation(Quat::from_rotation_x(
                    -std::f32::consts::FRAC_PI_2,
                )),
                ..default()
            },
            HoverMarker,
        ));
    }
}

pub fn auto_orient_entire_map(map: &mut TileMap) -> bool {
    let mut changed = false;
    for y in 0..map.height {
        for x in 0..map.width {
            changed |= auto_orient_tile(map, x, y);
        }
    }
    changed
}

fn auto_orient_around(map: &mut TileMap, x: u32, y: u32) -> bool {
    let mut changed = false;
    let mut consider = |tx: i32, ty: i32| {
        if tx < 0 || ty < 0 {
            return;
        }
        let (ux, uy) = (tx as u32, ty as u32);
        if ux >= map.width || uy >= map.height {
            return;
        }
        changed |= auto_orient_tile(map, ux, uy);
    };

    consider(x as i32, y as i32);
    consider(x as i32 - 1, y as i32);
    consider(x as i32 + 1, y as i32);
    consider(x as i32, y as i32 - 1);
    consider(x as i32, y as i32 + 1);

    changed
}

fn auto_orient_tile(map: &mut TileMap, x: u32, y: u32) -> bool {
    let idx = map.idx(x, y);
    let tile = map.tiles[idx].clone();
    if tile.kind != TileKind::Ramp || tile.manual_orientation {
        return false;
    }

    if let Some(orientation) = best_orientation(map, x, y) {
        if tile.orientation != orientation {
            map.tiles[idx].orientation = orientation;
            return true;
        }
    }

    false
}

fn best_orientation(map: &TileMap, x: u32, y: u32) -> Option<Orientation4> {
    const EPS: f32 = 1e-4;
    let base = map.get(x, y).elevation as f32 * TILE_HEIGHT;
    let mut best: Option<(Orientation4, f32)> = None;

    for orientation in [
        Orientation4::North,
        Orientation4::East,
        Orientation4::South,
        Orientation4::West,
    ] {
        let (dx, dy) = match orientation {
            Orientation4::North => (0, -1),
            Orientation4::East => (1, 0),
            Orientation4::South => (0, 1),
            Orientation4::West => (-1, 0),
        };
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
        let h = neighbor.elevation as f32 * TILE_HEIGHT;
        if h > base {
            let diff = h - base;
            match best {
                None => best = Some((orientation, diff)),
                Some((_, best_diff)) => {
                    if diff + EPS < best_diff {
                        best = Some((orientation, diff));
                    }
                }
            }
        }
    }

    best.map(|(orientation, _)| orientation)
}
