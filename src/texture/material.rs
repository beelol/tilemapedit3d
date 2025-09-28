use bevy::math::Vec4;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};

use crate::types::{TERRAIN_LAYERS, TileType};

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
    #[sampler(102)]
    pub splatmap: Handle<Image>,
    #[texture(103)]
    #[sampler(104)]
    pub layer0: Handle<Image>,
    #[texture(105)]
    #[sampler(106)]
    pub layer1: Handle<Image>,
    #[texture(107)]
    #[sampler(108)]
    pub layer2: Handle<Image>,
    #[texture(109)]
    #[sampler(110)]
    pub layer3: Handle<Image>,
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
