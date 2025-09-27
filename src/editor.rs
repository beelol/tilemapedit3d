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
}
impl Default for EditorState {
    fn default() -> Self {
        Self {
            current_kind: TileKind::Floor,
            current_elev: 0,
            hover: None,
            map: TileMap::new(64, 64),
            map_dirty: true,
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
    if buttons.pressed(MouseButton::Left) {
        if let Some((x, y)) = state.hover {
            let kind = state.current_kind;
            let elevation = state.current_elev;
            let state_ref = &mut *state;
            let current = state_ref.map.get(x, y);
            if current.kind != kind || current.elevation != elevation {
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
                    },
                );
                state_ref.map_dirty = true;
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
