use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;

use crate::terrain::TerrainUvSettings;

pub type TerrainMaterial = ExtendedMaterial<StandardMaterial, TerrainMaterialExtension>;

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct TerrainMaterialExtension {
    #[uniform(100)]
    pub uv_scale: f32,
}

impl MaterialExtension for TerrainMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain_material.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/terrain_material.wgsl".into()
    }
}

#[derive(Debug, Clone)]
pub struct TerrainMaterialHandles {
    pub material: Handle<TerrainMaterial>,
    pub base_color: Handle<Image>,
    pub normal: Option<Handle<Image>>,
    pub roughness: Option<Handle<Image>>,
    pub specular: Option<Handle<Image>>,
}

/// Load a terrain material and keep the individual texture handles around so they can be
/// reused for things like UI previews.
pub fn load_terrain_material(
    asset_server: &AssetServer,
    materials: &mut Assets<TerrainMaterial>,
    base_color: String,
    normal: Option<String>,
    roughness: Option<String>,
    specular: Option<String>,
    uv_scale: f32,
) -> TerrainMaterialHandles {
    let base_color_handle: Handle<Image> = asset_server.load(base_color);
    let normal_handle: Option<Handle<Image>> = normal.map(|path| asset_server.load(path));
    let roughness_handle: Option<Handle<Image>> = roughness.map(|path| asset_server.load(path));
    let specular_handle: Option<Handle<Image>> = specular.map(|path| asset_server.load(path));

    let mut base = StandardMaterial {
        base_color_texture: Some(base_color_handle.clone()),
        normal_map_texture: normal_handle.clone(),
        metallic_roughness_texture: specular_handle.clone(),
        occlusion_texture: roughness_handle.clone(),
        ..default()
    };

    base.perceptual_roughness = 1.0;
    base.metallic = 0.0;

    let material_handle = materials.add(TerrainMaterial {
        base,
        extension: TerrainMaterialExtension { uv_scale },
    });

    TerrainMaterialHandles {
        material: material_handle,
        base_color: base_color_handle,
        normal: normal_handle,
        roughness: roughness_handle,
        specular: specular_handle,
    }
}

pub fn sync_material_uv_scale(
    settings: Option<Res<TerrainUvSettings>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let Some(settings) = settings else {
        return;
    };

    if !settings.is_changed() {
        return;
    }

    let scale = settings.uv_scale();
    for (_, material) in materials.iter_mut() {
        material.extension.uv_scale = scale;
    }
}
