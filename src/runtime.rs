use crate::editor::EditorState;
use crate::terrain::{self, TerrainMeshSet};
use crate::texture::material::{self, TerrainMaterial};
use crate::texture::registry::TerrainTextureRegistry;
use crate::types::TileType;
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
    pub needs_rebuild: bool,
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
        needs_rebuild: true,
    });
}

fn rebuild_runtime_mesh(
    state: Res<EditorState>,
    mut runtime: Option<ResMut<RuntimeTerrainVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(mut runtime) = runtime else {
        return;
    };

    runtime.needs_rebuild |= state.map_dirty;

    if !runtime.needs_rebuild {
        return;
    }

    let combined = terrain::build_combined_mesh(&state.map);

    if let Some(existing) = meshes.get_mut(&runtime.mesh) {
        *existing = combined;
    }

    runtime.needs_rebuild = false;
}

fn update_runtime_material(
    mut textures: ResMut<TerrainTextureRegistry>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    runtime: Option<Res<RuntimeTerrainVisual>>,
) {
    let Some(runtime) = runtime else {
        return;
    };

    let Some(material) = materials.get_mut(&runtime.material) else {
        return;
    };

    let Some(array_handle) = textures.ensure_texture_array(&mut images) else {
        return;
    };

    let desired_layers = images
        .get(&array_handle)
        .map(|image| image.texture_descriptor.size.depth_or_array_layers)
        .unwrap_or(0);

    if desired_layers == 0 {
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
}
