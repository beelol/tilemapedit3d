use bevy::asset::Asset;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

use crate::types::TILE_SIZE;

pub type TerrainMaterial = ExtendedMaterial<StandardMaterial, TerrainMaterialExtension>;

#[derive(Debug, Clone)]
pub struct TerrainMaterialHandles {
    pub material: Handle<TerrainMaterial>,
    pub base_color: Handle<Image>,
    pub normal: Option<Handle<Image>>,
    pub roughness: Option<Handle<Image>>,
    pub specular: Option<Handle<Image>>,
}

const TILE_REPEAT: f32 = 4.0;

fn default_uv_scale() -> f32 {
    1.0 / (TILE_SIZE * TILE_REPEAT)
}

#[derive(Asset, AsBindGroup, Debug, Clone, TypePath)]
pub struct TerrainMaterialExtension {
    #[uniform(100)]
    pub uv_scale: f32,
}

impl Default for TerrainMaterialExtension {
    fn default() -> Self {
        Self {
            uv_scale: default_uv_scale(),
        }
    }
}

impl MaterialExtension for TerrainMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain_pbr_extension.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/terrain_pbr_extension.wgsl".into()
    }
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
) -> TerrainMaterialHandles {
    let base_color_handle: Handle<Image> = asset_server.load(base_color);
    let normal_handle: Option<Handle<Image>> = normal.map(|path| asset_server.load(path));
    let roughness_handle: Option<Handle<Image>> = roughness.map(|path| asset_server.load(path));
    let specular_handle: Option<Handle<Image>> = specular.map(|path| asset_server.load(path));

    let mut base_material = StandardMaterial {
        base_color_texture: Some(base_color_handle.clone()),
        normal_map_texture: normal_handle.clone(),
        metallic_roughness_texture: specular_handle.clone(),
        occlusion_texture: roughness_handle.clone(),
        ..default()
    };

    base_material.perceptual_roughness = 1.0;
    base_material.metallic = 0.0;

    let material_handle = materials.add(TerrainMaterial {
        base: base_material,
        extension: TerrainMaterialExtension::default(),
    });

    TerrainMaterialHandles {
        material: material_handle,
        base_color: base_color_handle,
        normal: normal_handle,
        roughness: roughness_handle,
        specular: specular_handle,
    }
}
