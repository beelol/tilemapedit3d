use bevy::math::Vec3;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial};
use bevy::prelude::*;
use bevy::render::render_resource::{ShaderRef, ShaderType};

pub const DEFAULT_TILES_PER_TEXTURE: f32 = 4.0;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct TerrainMaterialExtension {
    #[uniform(15)]
    pub params: TerrainMaterialUniform,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct TerrainMaterialUniform {
    pub tiles_per_texture: f32,
    pub _padding: Vec3,
}

impl Default for TerrainMaterialExtension {
    fn default() -> Self {
        Self {
            params: TerrainMaterialUniform {
                tiles_per_texture: DEFAULT_TILES_PER_TEXTURE,
                _padding: Vec3::ZERO,
            },
        }
    }
}

impl TerrainMaterialExtension {
    pub fn set_tiles_per_texture(&mut self, tiles_per_texture: f32) {
        self.params.tiles_per_texture = tiles_per_texture.max(0.0001);
    }
}

impl MaterialExtension for TerrainMaterialExtension {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Path("shaders/terrain_world_uv.wgsl".into())
    }
}

pub type TerrainMaterial = ExtendedMaterial<StandardMaterial, TerrainMaterialExtension>;

#[derive(Resource, Debug, Clone)]
pub struct TerrainMaterialSettings {
    pub tiles_per_texture: f32,
}

impl Default for TerrainMaterialSettings {
    fn default() -> Self {
        Self {
            tiles_per_texture: DEFAULT_TILES_PER_TEXTURE,
        }
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
    settings: &TerrainMaterialSettings,
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

    let mut extension = TerrainMaterialExtension::default();
    extension.set_tiles_per_texture(settings.tiles_per_texture);

    let material_handle = materials.add(TerrainMaterial {
        base: base_material,
        extension,
    });

    TerrainMaterialHandles {
        material: material_handle,
        base_color: base_color_handle,
        normal: normal_handle,
        roughness: roughness_handle,
        specular: specular_handle,
    }
}
