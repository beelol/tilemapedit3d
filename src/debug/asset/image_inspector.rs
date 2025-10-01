use bevy::prelude::*;
use bevy::render::texture::Image;

pub struct ImageInspectorPlugin;

impl Plugin for ImageInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, inspect_loaded_images);
    }
}

fn inspect_loaded_images(assets: Res<Assets<Image>>, mut events: EventReader<AssetEvent<Image>>) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            if let Some(image) = assets.get(*id) {
                info!("--- Loaded Image ---");
                info!("Size: {:?}", image.texture_descriptor.size);
                info!("Format: {:?}", image.texture_descriptor.format);
                info!("Usage: {:?}", image.texture_descriptor.usage);
                info!("Mip levels: {}", image.texture_descriptor.mip_level_count);
                info!(
                    "Array layers: {}",
                    image.texture_descriptor.array_layer_count()
                );
                info!("Dimension: {:?}", image.texture_descriptor.dimension);

                // check sRGB
                let srgb = image.texture_descriptor.format.is_srgb();
                info!("is_srgb: {}", srgb);

                // preview a few bytes
                let bytes = &image.data[..16.min(image.data.len())];
                info!("First few bytes: {:?}", bytes);
            }
        }
    }
}
