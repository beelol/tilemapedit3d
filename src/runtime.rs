use crate::editor::EditorState;
use crate::terrain::{self, TerrainMeshSet};
use crate::texture::material::{self, TerrainMaterial};
use crate::texture::registry::TerrainTextureRegistry;
use crate::types::TileType;
use bevy::asset::{AssetId, LoadState};
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
            waiting_for_textures |= check_handle_state(
                &asset_server,
                entry.preview.id(),
                entry.tile_type,
                &mut encountered_failure,
                "Terrain preview texture failed to load",
            );

            if let Some(normal) = entry.normal.as_ref() {
                waiting_for_textures |= check_handle_state(
                    &asset_server,
                    normal.id(),
                    entry.tile_type,
                    &mut encountered_failure,
                    "Terrain normal map failed to load",
                );
            }

            if let Some(roughness) = entry.roughness.as_ref() {
                waiting_for_textures |= check_handle_state(
                    &asset_server,
                    roughness.id(),
                    entry.tile_type,
                    &mut encountered_failure,
                    "Terrain roughness map failed to load",
                );
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

    let Some((base_handle, normal_handle, roughness_handle)) =
        textures.ensure_texture_arrays(&mut images)
    else {
        error!("Failed to assemble terrain texture arrays after previews loaded");
        *visibility = Visibility::Hidden;
        return;
    };

    let desired_layers = images
        .get(&base_handle)
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
        .base_color_array
        .as_ref()
        .map(|handle| handle != &base_handle)
        .unwrap_or(true)
    {
        material.extension.base_color_array = Some(base_handle.clone());
    }

    match normal_handle {
        Some(handle) => {
            if material
                .extension
                .normal_array
                .as_ref()
                .map(|existing| existing != &handle)
                .unwrap_or(true)
            {
                material.extension.normal_array = Some(handle.clone());
            }
        }
        None => {
            material.extension.normal_array = None;
        }
    }

    match roughness_handle {
        Some(handle) => {
            if material
                .extension
                .roughness_array
                .as_ref()
                .map(|existing| existing != &handle)
                .unwrap_or(true)
            {
                material.extension.roughness_array = Some(handle.clone());
            }
        }
        None => {
            material.extension.roughness_array = None;
        }
    }

    *visibility = Visibility::Visible;
}

fn check_handle_state(
    asset_server: &AssetServer,
    id: AssetId<Image>,
    tile_type: TileType,
    encountered_failure: &mut bool,
    message: &str,
) -> bool {
    match asset_server.get_load_state(id) {
        Some(LoadState::Loaded) => false,
        Some(LoadState::Failed(_)) => {
            error!(tile_type = ?tile_type, message);
            *encountered_failure = true;
            false
        }
        _ => true,
    }
}
