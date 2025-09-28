use bevy::prelude::*;

pub mod material;
pub mod registry;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<registry::TerrainTextureRegistry>()
            .add_plugins(material::TerrainMaterialPlugin);
    }
}
