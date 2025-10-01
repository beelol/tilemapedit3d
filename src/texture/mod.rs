use bevy::asset::AssetEvent;
use bevy::ecs::schedule::common_conditions::on_event;
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;
use bevy::render::texture::Image;

pub mod material;
pub mod registry;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<material::TerrainMaterial>::default())
            .init_resource::<registry::TerrainTextureRegistry>()
            .add_systems(
                Update,
                material::format_loaded_terrain_maps.run_if(on_event::<AssetEvent<Image>>()),
            );
    }
}
