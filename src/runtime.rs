use crate::editor::EditorState;
use crate::terrain::{self, TerrainMeshSet, splatmap};
use crate::texture::material::{self, TerrainMaterial};
use crate::texture::registry::TerrainTextureRegistry;
use crate::types::{TILE_SIZE, TileType};
use bevy::asset::{AssetId, LoadState};
use bevy::math::{UVec2, Vec2};
use bevy::pbr::MaterialMeshBundle;
use bevy::prelude::*;
use bevy::render::texture::Image;

pub struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_runtime_mesh).add_systems(
            Update,
            (
                generate_splat_map,
                rebuild_runtime_mesh,
                update_runtime_material,
            )
                .chain()
                .in_set(TerrainMeshSet::Rebuild),
        );
    }
}

#[derive(Resource)]
pub struct RuntimeTerrainVisual {
    pub mesh: Handle<Mesh>,
    pub material: Handle<TerrainMaterial>,
    pub entity: Entity,
}

#[derive(Resource)]
pub struct RuntimeSplatMap {
    pub handle: Handle<Image>,
    pub size: UVec2,
}

fn setup_runtime_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    mut images: ResMut<Assets<Image>>,
    state: Res<EditorState>,
) {
    let mesh = meshes.add(terrain::empty_mesh());
    let material = material::create_runtime_material(&mut materials);
    let splat_image = splatmap::create(&state.map);
    let splat_handle = images.add(splat_image);
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
    commands.insert_resource(RuntimeSplatMap {
        handle: splat_handle,
        size: UVec2::new(state.map.width.max(1), state.map.height.max(1)),
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

fn generate_splat_map(
    state: Res<EditorState>,
    mut runtime_splat: Option<ResMut<RuntimeSplatMap>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !state.map_dirty {
        return;
    }

    let Some(mut runtime_splat) = runtime_splat else {
        return;
    };

    let Some(image) = images.get_mut(&runtime_splat.handle) else {
        return;
    };

    splatmap::write(&state.map, image);
    runtime_splat.size = UVec2::new(state.map.width.max(1), state.map.height.max(1));
}

fn update_runtime_material(
    mut textures: ResMut<TerrainTextureRegistry>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    asset_server: Res<AssetServer>,
    runtime: Option<Res<RuntimeTerrainVisual>>,
    mut visibility_query: Query<&mut Visibility>,
    splat: Option<Res<RuntimeSplatMap>>,
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

        if let Some(wall) = registry.wall_texture() {
            waiting_for_textures |= check_image_handle_state(
                &asset_server,
                wall.base_color.id(),
                &mut encountered_failure,
                "Wall base color texture failed to load",
            );

            if let Some(normal) = wall.normal.as_ref() {
                waiting_for_textures |= check_image_handle_state(
                    &asset_server,
                    normal.id(),
                    &mut encountered_failure,
                    "Wall normal map failed to load",
                );
            }

            if let Some(roughness) = wall.roughness.as_ref() {
                waiting_for_textures |= check_image_handle_state(
                    &asset_server,
                    roughness.id(),
                    &mut encountered_failure,
                    "Wall roughness map failed to load",
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

    let Some(splat) = splat else {
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

    if material
        .extension
        .splat_map
        .as_ref()
        .map(|existing| existing != &splat.handle)
        .unwrap_or(true)
    {
        material.extension.splat_map = Some(splat.handle.clone());
    }

    let has_wall_texture = textures.wall_texture().is_some();
    let wall_arrays = if has_wall_texture {
        textures.ensure_wall_texture_arrays(&mut images)
    } else {
        None
    };

    if has_wall_texture {
        let Some((wall_base_color, wall_normal, wall_roughness)) = wall_arrays else {
            error!("Failed to assemble wall texture array after wall textures loaded");
            *visibility = Visibility::Hidden;
            return;
        };

        if material
            .extension
            .wall_base_color
            .as_ref()
            .map(|existing| existing != &wall_base_color)
            .unwrap_or(true)
        {
            material.extension.wall_base_color = Some(wall_base_color.clone());
        }

        match wall_normal {
            Some(handle) => {
                if material
                    .extension
                    .wall_normal
                    .as_ref()
                    .map(|existing| existing != &handle)
                    .unwrap_or(true)
                {
                    material.extension.wall_normal = Some(handle.clone());
                }
            }
            None => {
                material.extension.wall_normal = None;
            }
        }

        match wall_roughness {
            Some(handle) => {
                if material
                    .extension
                    .wall_roughness
                    .as_ref()
                    .map(|existing| existing != &handle)
                    .unwrap_or(true)
                {
                    material.extension.wall_roughness = Some(handle.clone());
                }
            }
            None => {
                material.extension.wall_roughness = None;
            }
        }
    } else {
        material.extension.wall_base_color = None;
        material.extension.wall_normal = None;
        material.extension.wall_roughness = None;
    }

    material.extension.params.map_size = Vec2::new(splat.size.x as f32, splat.size.y as f32);
    material.extension.params.tile_size = TILE_SIZE;
    material.extension.params.cliff_blend_height = 0.2;

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

fn check_image_handle_state(
    asset_server: &AssetServer,
    id: AssetId<Image>,
    encountered_failure: &mut bool,
    message: &str,
) -> bool {
    match asset_server.get_load_state(id) {
        Some(LoadState::Loaded) => false,
        Some(LoadState::Failed(_)) => {
            error!("{message}");
            *encountered_failure = true;
            false
        }
        _ => true,
    }
}
