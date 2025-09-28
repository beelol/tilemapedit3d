use std::collections::HashMap;

use bevy::math::Vec4;
use bevy::prelude::*;

use crate::types::{TERRAIN_LAYERS, TileType};

#[derive(Debug, Clone)]
pub struct TerrainTextureEntry {
    pub tile_type: TileType,
    pub name: String,
    pub icon: Handle<Image>,
    pub base_color: Handle<Image>,
    pub tint: Color,
}

#[derive(Resource, Default)]
pub struct TerrainTextureRegistry {
    entries: Vec<TerrainTextureEntry>,
    lookup: HashMap<TileType, usize>,
}

impl TerrainTextureRegistry {
    pub fn register_loaded(&mut self, entry: TerrainTextureEntry) {
        if let Some(index) = self.lookup.get(&entry.tile_type).copied() {
            self.entries[index] = entry;
        } else {
            let index = self.entries.len();
            self.lookup.insert(entry.tile_type, index);
            self.entries.push(entry);
        }
    }

    pub fn load_and_register(
        &mut self,
        tile_type: TileType,
        name: impl Into<String>,
        asset_server: &AssetServer,
        base_color: &'static str,
        tint: Color,
    ) -> Handle<Image> {
        let base_color_handle: Handle<Image> = asset_server.load(base_color);

        self.register_loaded(TerrainTextureEntry {
            tile_type,
            name: name.into(),
            icon: base_color_handle.clone(),
            base_color: base_color_handle.clone(),
            tint,
        });

        base_color_handle
    }

    pub fn register_from_handle(
        &mut self,
        tile_type: TileType,
        name: impl Into<String>,
        base_color: Handle<Image>,
        tint: Color,
    ) {
        self.register_loaded(TerrainTextureEntry {
            tile_type,
            name: name.into(),
            icon: base_color.clone(),
            base_color,
            tint,
        });
    }

    pub fn iter(&self) -> impl Iterator<Item = &TerrainTextureEntry> {
        self.entries.iter()
    }

    pub fn get(&self, tile_type: TileType) -> Option<&TerrainTextureEntry> {
        self.lookup
            .get(&tile_type)
            .and_then(|index| self.entries.get(*index))
    }

    pub fn layer_textures(
        &self,
    ) -> (
        [Handle<Image>; TERRAIN_LAYERS.len()],
        [Vec4; TERRAIN_LAYERS.len()],
    ) {
        let mut handles: [Handle<Image>; TERRAIN_LAYERS.len()] = Default::default();
        let mut tints = [Vec4::ONE; TERRAIN_LAYERS.len()];

        for (index, layer) in TERRAIN_LAYERS.iter().enumerate() {
            if let Some(entry) = self.get(*layer) {
                handles[index] = entry.base_color.clone();
                let tint_linear = entry.tint.to_linear();
                tints[index] = Vec4::new(
                    tint_linear.red,
                    tint_linear.green,
                    tint_linear.blue,
                    tint_linear.alpha,
                );
            }
        }

        (handles, tints)
    }
}
