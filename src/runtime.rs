use crate::editor::EditorState;
use crate::terrain::{self, TerrainMeshSet};
use crate::texture::material::{self, TerrainMaterial};
use crate::texture::registry::TerrainTextureRegistry;
use crate::types::TileType;
use bevy::asset::LoadState;
use bevy::pbr::MaterialMeshBundle;
use bevy::prelude::*;

pub struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_runtime_mesh).add_systems(
            Update,
            (
                rebuild_runtime_mesh.in_set(TerrainMeshSet::Rebuild),
                update_runtime_material.in_set(TerrainMeshSet::Rebuild),
            ),
        );
    }
}

#[derive(Resource)]
pub struct RuntimeTerrainVisual {
    pub mesh: Handle<Mesh>,
    pub material: Handle<TerrainMaterial>,
    pub entity: Entity,
}

fn setup_runtime_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let mesh = meshes.add(terrain::empty_mesh());
    let material = material::create_runtime_material(&mut materials);
    let entity = commands
        .spawn((
            MaterialMeshBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::default(),
                visibility: Visibility::Visible,
                ..default()
            },
            Name::new("RuntimeTerrain"),
        ))
        .id();

    commands.insert_resource(RuntimeTerrainVisual {
        mesh,
        material,
        entity,
    });
}

fn rebuild_runtime_mesh(
    state: Res<EditorState>,
    runtime: Option<Res<RuntimeTerrainVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !state.map_dirty {
        return;
    }

    let Some(runtime) = runtime else {
        return;
    };

    let combined = terrain::build_combined_mesh(&state.map);

    if let Some(existing) = meshes.get_mut(&runtime.mesh) {
        *existing = combined;
    }
}

fn update_runtime_material(
    mut textures: ResMut<TerrainTextureRegistry>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    asset_server: Res<AssetServer>,
    runtime: Option<Res<RuntimeTerrainVisual>>,
    mut visibility_query: Query<&mut Visibility>,
) {
    let Some(runtime) = runtime else {
        return;
    };

    let Ok(mut visibility) = visibility_query.get_mut(runtime.entity) else {
        return;
    };

    let mut waiting_for_textures = false;
    let mut encountered_failure = false;

    {
        let registry = textures.as_ref();
        for entry in registry.iter() {
            match asset_server.get_load_state(entry.preview.id()) {
                Some(LoadState::Loaded) => {}
                Some(LoadState::Failed(_)) => {
                    error!(
                        tile_type = ?entry.tile_type,
                        "Terrain preview texture failed to load"
                    );
                    encountered_failure = true;
                }
                _ => {
                    waiting_for_textures = true;
                }
            }
        }
    }

    if encountered_failure {
        *visibility = Visibility::Hidden;
        return;
    }

    if waiting_for_textures {
        *visibility = Visibility::Hidden;
        return;
    }

    let Some(material) = materials.get_mut(&runtime.material) else {
        *visibility = Visibility::Hidden;
        return;
    };

    let Some(array_handle) = textures.ensure_texture_array(&mut images) else {
        error!("Failed to assemble terrain texture array after previews loaded");
        *visibility = Visibility::Hidden;
        return;
    };

    let desired_layers = images
        .get(&array_handle)
        .map(|image| image.texture_descriptor.size.depth_or_array_layers)
        .unwrap_or(0);

    if desired_layers == 0 {
        *visibility = Visibility::Hidden;
        return;
    }

    if material.extension.params.layer_count != desired_layers {
        material.extension.params.layer_count = desired_layers;
    }

    if material
        .extension
        .texture_array
        .as_ref()
        .map(|handle| handle != &array_handle)
        .unwrap_or(true)
    {
        material.extension.texture_array = Some(array_handle.clone());
    }

    *visibility = Visibility::Visible;
}
