mod material;

use bevy::prelude::*;
use bevy::pbr::StandardMaterial;

/// Holds all the textures for a terrain material
#[derive(Debug, Clone)]
pub struct TerrainMaterial {
    pub base_color: Handle<Image>,
    pub normal: Option<Handle<Image>>,
    pub roughness: Option<Handle<Image>>,
}

impl TerrainMaterial {
    /// Create a new terrain StandardMaterial
    pub fn to_standard(&self) -> StandardMaterial {
        let mut mat = StandardMaterial {
            base_color_texture: Some(self.base_color.clone()),
            normal_map_texture: self.normal.clone(),
            metallic_roughness_texture: self.roughness.clone(),
            ..default()
        };

        // Optional tweaks for terrain look
        mat.perceptual_roughness = 1.0; // default to matte
        mat.metallic = 0.0;             // terrain is not metal
        mat
    }
}


/// Utility to build a terrain material with optional maps.
/// Reusable for dirt, grass, rocky, snow, etc.
pub fn load_terrain_material(
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    base_color: &str,
    normal: Option<&str>,
    roughness: Option<&str>,
    specular: Option<&str>,
) -> Handle<StandardMaterial> {
    let mut mat = StandardMaterial {
        base_color_texture: Some(asset_server.load(base_color)),
        ..default()
    };

    if let Some(path) = normal {
        mat.normal_map_texture = Some(asset_server.load(path));
    }
    if let Some(path) = roughness {
        mat.occlusion_texture = Some(asset_server.load(path));
    }
    if let Some(path) = specular {
        mat.metallic_roughness_texture = Some(asset_server.load(path));
    }

    materials.add(mat)
}
