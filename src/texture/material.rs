use bevy::asset::Asset;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup, ShaderRef, ShaderType, TextureDimension, TextureViewDescriptor,
    TextureViewDimension,
};

use crate::types::{TILE_SIZE, TileType};

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

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct TerrainMaterialParams {
    pub uv_scale: f32,
    pub tile_type_override: i32,
    pub layer_count: u32,
    pub _pad: u32,
}

#[derive(Asset, AsBindGroup, Debug, Clone, TypePath)]
pub struct TerrainMaterialExtension {
    #[uniform(100)]
    pub params: TerrainMaterialParams,
    #[texture(101, dimension = "2d_array")]
    #[sample_type = "Float { filterable: true }"]
    pub base_color_array: Handle<Image>,
}

impl Default for TerrainMaterialExtension {
    fn default() -> Self {
        Self {
            params: TerrainMaterialParams {
                uv_scale: default_uv_scale(),
                tile_type_override: -1,
                layer_count: 0,
                _pad: 0,
            },
            base_color_array: Handle::default(),
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

#[derive(Resource, Default, Debug, Clone)]
pub struct TerrainTextureArray {
    pub base_color: Option<Handle<Image>>,
    pub layer_count: u32,
}

pub fn set_material_tile_override(
    materials: &mut Assets<TerrainMaterial>,
    handle: &Handle<TerrainMaterial>,
    tile_type: TileType,
) {
    if let Some(material) = materials.get_mut(handle) {
        material.extension.params.tile_type_override = tile_type.as_index() as i32;
    }
}

pub fn configure_runtime_material(
    materials: &mut Assets<TerrainMaterial>,
) -> Handle<TerrainMaterial> {
    let mut base_material = StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    };
    base_material.perceptual_roughness = 1.0;
    base_material.metallic = 0.0;

    materials.add(TerrainMaterial {
        base: base_material,
        extension: TerrainMaterialExtension::default(),
    })
}

pub fn build_texture_array_image(layers: &[&Image]) -> Option<Image> {
    let first = layers.first()?;

    let mut array_image = first.clone();
    array_image.data.clear();

    for image in layers {
        if image.texture_descriptor.size != first.texture_descriptor.size
            || image.texture_descriptor.mip_level_count != first.texture_descriptor.mip_level_count
            || image.texture_descriptor.format != first.texture_descriptor.format
        {
            return None;
        }

        array_image.data.extend_from_slice(&image.data);
    }

    let layer_count = layers.len() as u32;
    array_image.texture_descriptor.size.depth_or_array_layers = layer_count;
    array_image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::D2Array),
        ..Default::default()
    });
    array_image.texture_descriptor.dimension = TextureDimension::D2;
    array_image.sampler = first.sampler.clone();
    array_image.asset_usage = first.asset_usage;

    Some(array_image)
}
