use bevy::math::Vec4;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};
use bevy::render::texture::ImageSampler;

use crate::types::TERRAIN_LAYERS;

pub type TerrainMaterial = ExtendedMaterial<StandardMaterial, TerrainBlendExtension>;

pub struct TerrainMaterialPlugin;

impl Plugin for TerrainMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<TerrainMaterial>::default());
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct TerrainBlendParams {
    pub uv_scale: f32,
    pub inv_map_width: f32,
    pub inv_map_height: f32,
    pub _padding: f32,
    pub layer_tints: [Vec4; TERRAIN_LAYERS.len()],
}

#[derive(Asset, AsBindGroup, Clone, Debug, TypePath)]
pub struct TerrainBlendExtension {
    #[uniform(100)]
    pub params: TerrainBlendParams,
    #[texture(101)]
    pub splatmap: Handle<Image>,
    #[sampler(102)]
    pub splatmap_sampler: ImageSampler,
    #[texture(103)]
    pub layer0: Handle<Image>,
    #[texture(104)]
    pub layer1: Handle<Image>,
    #[texture(105)]
    pub layer2: Handle<Image>,
    #[texture(106)]
    pub layer3: Handle<Image>,
    #[sampler(107)]
    pub layer_sampler: ImageSampler,
}

impl TerrainBlendExtension {
    pub fn set_layer_handles(&mut self, handles: &[Handle<Image>; TERRAIN_LAYERS.len()]) {
        self.layer0 = handles[0].clone();
        self.layer1 = handles[1].clone();
        self.layer2 = handles[2].clone();
        self.layer3 = handles[3].clone();
    }
}

impl MaterialExtension for TerrainBlendExtension {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path("shaders/terrain_blend_extension.wgsl".into())
    }

    fn deferred_fragment_shader() -> ShaderRef {
        ShaderRef::Path("shaders/terrain_blend_extension.wgsl".into())
    }
}
