use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};

/// Number of terrain texture layers supported by the splat material.
pub const TERRAIN_LAYER_COUNT: usize = 4;

/// Uniform parameters for the terrain splat shader.
#[derive(Clone, Copy, Debug, ShaderType)]
pub struct TerrainSplatSettings {
    /// Size of the terrain in world units (x = width, y = height).
    pub map_size: Vec2,
    /// Scaling factor used to convert world coordinates into UV space.
    pub uv_scale: f32,
    /// Padding to keep the structure 16-byte aligned for WGSL.
    pub _padding: f32,
}

impl Default for TerrainSplatSettings {
    fn default() -> Self {
        Self {
            map_size: Vec2::ONE,
            uv_scale: 4.0,
            _padding: 0.0,
        }
    }
}

/// Extension that blends up to four terrain textures using a splatmap.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerrainSplatExtension {
    #[uniform(100)]
    pub settings: TerrainSplatSettings,
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

impl Default for TerrainSplatExtension {
    fn default() -> Self {
        Self {
            settings: TerrainSplatSettings::default(),
            splatmap: Handle::default(),
            layer0: Handle::default(),
            layer1: Handle::default(),
            layer2: Handle::default(),
            layer3: Handle::default(),
        }
    }
}

impl MaterialExtension for TerrainSplatExtension {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path("shaders/terrain_splat.wgsl".into())
    }

    fn deferred_fragment_shader() -> ShaderRef {
        ShaderRef::Path("shaders/terrain_splat.wgsl".into())
    }
}

/// Convenience alias for the extended terrain material used by the editor.
pub type TerrainMaterial = ExtendedMaterial<StandardMaterial, TerrainSplatExtension>;
