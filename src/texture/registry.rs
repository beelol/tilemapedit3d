use std::collections::HashMap;

use bevy::prelude::*;

use crate::types::TileType;

use super::material::{self, TerrainMaterial, TerrainMaterialHandles};

#[derive(Debug, Clone)]
pub struct TerrainTextureEntry {
    pub tile_type: TileType,
    pub name: String,
    pub preview: Handle<Image>,
    pub material: Handle<TerrainMaterial>,
}

#[derive(Resource, Default)]
pub struct TerrainTextureRegistry {
    entries: Vec<TerrainTextureEntry>,
    lookup: HashMap<TileType, usize>,
    texture_array: Option<Handle<Image>>,
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
        self.texture_array = None;
    }

    pub fn load_and_register(
        &mut self,
        tile_type: TileType,
        name: impl Into<String>,
        asset_server: &AssetServer,
        materials: &mut Assets<TerrainMaterial>,
        base_color: &str,
        normal: Option<&str>,
        roughness: Option<&str>,
        specular: Option<&str>,
    ) -> Handle<TerrainMaterial> {
        let TerrainMaterialHandles {
            material,
            base_color: preview,
            ..
        } = material::load_terrain_material(
            asset_server,
            materials,
            base_color.to_string(),
            normal.map(|s| s.to_string()),
            roughness.map(|s| s.to_string()),
            specular.map(|s| s.to_string()),
        );

        self.register_loaded(TerrainTextureEntry {
            tile_type,
            name: name.into(),
            preview,
            material: material.clone(),
        });

        material
    }

    pub fn iter(&self) -> impl Iterator<Item = &TerrainTextureEntry> {
        self.entries.iter()
    }

    pub fn get(&self, tile_type: TileType) -> Option<&TerrainTextureEntry> {
        self.lookup
            .get(&tile_type)
            .and_then(|index| self.entries.get(*index))
    }

    pub fn ensure_texture_array(&mut self, images: &mut Assets<Image>) -> Option<Handle<Image>> {
        if let Some(handle) = self.texture_array.clone() {
            if images.get(&handle).is_some() {
                return Some(handle);
            }
            self.texture_array = None;
        }

        let mut layers: Vec<&Image> = Vec::with_capacity(TileType::ALL.len());
        for tile_type in TileType::ALL {
            let entry_index = *self.lookup.get(&tile_type)?;
            let entry = self.entries.get(entry_index)?;
            let image = images.get(&entry.preview)?;
            layers.push(image);
        }

        let array_image = material::create_texture_array_image(&layers)?;



        let handle = images.add(array_image);
        self.texture_array = Some(handle.clone());
        Some(handle)
    }
}
