use bevy::prelude::*;

pub mod material;
pub mod registry;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<material::TerrainMaterial>::default())
            .init_resource::<registry::TerrainTextureRegistry>()
            .add_systems(Update, material::sync_material_uv_scale);
    }
}
