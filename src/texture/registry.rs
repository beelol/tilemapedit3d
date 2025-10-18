use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{TextureDimension, TextureFormat};

use crate::types::TileType;

use super::material::{self, TerrainMaterial, TerrainMaterialHandles};

#[derive(Debug, Clone)]
pub struct TerrainTextureEntry {
    pub tile_type: TileType,
    pub name: String,
    pub preview: Handle<Image>,
    pub material: Handle<TerrainMaterial>,
    pub normal: Option<Handle<Image>>,
    pub roughness: Option<Handle<Image>>,
    pub dispersion: Option<Handle<Image>>,
    pub diffuse_path: String,
    pub normal_path: Option<String>,
    pub roughness_path: Option<String>,
    pub dispersion_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WallTextureEntry {
    pub id: String,
    pub name: String,
    pub base_color: Handle<Image>,
    pub normal: Option<Handle<Image>>,
    pub roughness: Option<Handle<Image>>,
    pub diffuse_path: String,
    pub normal_path: Option<String>,
    pub roughness_path: Option<String>,
}

#[derive(Resource, Default)]
pub struct TerrainTextureRegistry {
    entries: Vec<TerrainTextureEntry>,
    lookup: HashMap<TileType, usize>,
    base_color_array: Option<Handle<Image>>,
    normal_array: Option<Handle<Image>>,
    roughness_array: Option<Handle<Image>>,
    wall_texture: Option<WallTextureEntry>,
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
        self.base_color_array = None;
        self.normal_array = None;
        self.roughness_array = None;
    }

    pub fn register_wall_texture(&mut self, entry: WallTextureEntry) {
        self.wall_texture = Some(entry);
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
        dispersion: Option<&str>,
    ) -> Handle<TerrainMaterial> {
        let TerrainMaterialHandles {
            material,
            base_color: preview,
            normal: normal_handle,
            roughness: roughness_handle,
            dispersion: dispersion_handle,
        } = material::load_terrain_material(
            asset_server,
            materials,
            base_color.to_string(),
            normal.map(|s| s.to_string()),
            roughness.map(|s| s.to_string()),
            dispersion.map(|s| s.to_string()),
        );

        self.register_loaded(TerrainTextureEntry {
            tile_type,
            name: name.into(),
            preview,
            material: material.clone(),
            normal: normal_handle,
            roughness: roughness_handle,
            dispersion: dispersion_handle,
            diffuse_path: base_color.to_string(),
            normal_path: normal.map(|s| s.to_string()),
            roughness_path: roughness.map(|s| s.to_string()),
            dispersion_path: dispersion.map(|s| s.to_string()),
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

    pub fn load_and_register_wall(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        asset_server: &AssetServer,
        base_color: &str,
        normal: Option<&str>,
        roughness: Option<&str>,
    ) {
        let base_color_handle: Handle<Image> = asset_server.load(base_color);
        let normal_handle: Option<Handle<Image>> = normal.map(|path| asset_server.load(path));
        let roughness_handle: Option<Handle<Image>> = roughness.map(|path| asset_server.load(path));

        self.register_wall_texture(WallTextureEntry {
            id: id.into(),
            name: name.into(),
            base_color: base_color_handle,
            normal: normal_handle,
            roughness: roughness_handle,
            diffuse_path: base_color.to_string(),
            normal_path: normal.map(|s| s.to_string()),
            roughness_path: roughness.map(|s| s.to_string()),
        });
    }

    pub fn wall_texture(&self) -> Option<&WallTextureEntry> {
        self.wall_texture.as_ref()
    }

    pub fn ensure_texture_arrays(
        &mut self,
        images: &mut Assets<Image>,
    ) -> Option<(Handle<Image>, Option<Handle<Image>>, Option<Handle<Image>>)> {
        let base_color = ensure_base_color_array(
            &self.entries,
            &self.lookup,
            images,
            &mut self.base_color_array,
        )?;

        let normal = ensure_optional_array(
            &self.entries,
            &self.lookup,
            images,
            &mut self.normal_array,
            |entry| entry.normal.as_ref(),
            [0.5, 0.5, 1.0, 1.0],
        );

        let roughness = ensure_optional_array(
            &self.entries,
            &self.lookup,
            images,
            &mut self.roughness_array,
            |entry| entry.roughness.as_ref(),
            [1.0, 1.0, 1.0, 1.0],
        );

        Some((base_color, normal, roughness))
    }
}

fn ensure_base_color_array(
    entries: &[TerrainTextureEntry],
    lookup: &HashMap<TileType, usize>,
    images: &mut Assets<Image>,
    cache: &mut Option<Handle<Image>>,
) -> Option<Handle<Image>> {
    if let Some(handle) = cache.clone() {
        if images.get(&handle).is_some() {
            return Some(handle);
        }
        *cache = None;
    }

    let mut layers: Vec<&Image> = Vec::with_capacity(TileType::ALL.len());
    for tile_type in TileType::ALL {
        let entry_index = *lookup.get(&tile_type)?;
        let entry = entries.get(entry_index)?;
        let image = images.get(&entry.preview)?;
        layers.push(image);
    }

    let array_image = material::create_texture_array_image(&layers)?;
    let handle = images.add(array_image);
    *cache = Some(handle.clone());
    Some(handle)
}

fn ensure_optional_array<F>(
    entries: &[TerrainTextureEntry],
    lookup: &HashMap<TileType, usize>,
    images: &mut Assets<Image>,
    cache: &mut Option<Handle<Image>>,
    accessor: F,
    fallback_color: [f32; 4],
) -> Option<Handle<Image>>
where
    F: Fn(&TerrainTextureEntry) -> Option<&Handle<Image>>,
{
    // check cache
    if let Some(handle) = cache.clone() {
        if images.get(&handle).is_some() {
            return Some(handle);
        }
        *cache = None;
    }

    // early exit if no textures at all
    let mut has_texture = false;
    for tile_type in TileType::ALL {
        let entry_index = *lookup.get(&tile_type)?;
        let entry = entries.get(entry_index)?;
        if accessor(entry).is_some() {
            has_texture = true;
            break;
        }
    }
    if !has_texture {
        return None;
    }

    // pick a template image for fallbacks
    let Some(template_image) = find_template_image(entries, lookup, images, &accessor) else {
        warn!("Skipping optional terrain texture array due to missing loaded source images");
        return None;
    };
    // Clone the template image to avoid borrowing conflicts
    let template_image_clone = template_image.clone();

    // --- pass 1: resolve handles (may mutate images) ---
    let mut handles: Vec<Handle<Image>> = Vec::with_capacity(TileType::ALL.len());
    for tile_type in TileType::ALL {
        let entry_index = *lookup.get(&tile_type)?;
        let entry = entries.get(entry_index)?;

        if let Some(handle) = accessor(entry) {
            // only record handle, check later
            handles.push(handle.clone());
        } else {
            let Some(fallback) = create_fallback_image(&template_image_clone, fallback_color)
            else {
                warn!("Skipping optional terrain texture array due to unsupported format");
                return None;
            };
            let fb_handle = images.add(fallback);
            handles.push(fb_handle);
        }
    }

    // --- pass 2: collect references immutably (no mutation here) ---
    let mut layers: Vec<&Image> = Vec::with_capacity(handles.len());
    for h in &handles {
        let Some(img) = images.get(h) else {
            return None;
        };
        layers.push(img);
    }

    // build the array image
    let array_image = material::create_texture_array_image(&layers)?;
    let handle = images.add(array_image);
    *cache = Some(handle.clone());
    Some(handle)
}

fn find_template_image<'a, F>(
    entries: &'a [TerrainTextureEntry],
    lookup: &HashMap<TileType, usize>,
    images: &'a bevy::prelude::Assets<bevy::prelude::Image>,
    accessor: &F,
) -> Option<&'a Image>
where
    F: Fn(&TerrainTextureEntry) -> Option<&Handle<Image>>,
{
    for tile_type in TileType::ALL {
        let entry_index = *lookup.get(&tile_type)?;
        let entry = entries.get(entry_index)?;
        if let Some(handle) = accessor(entry) {
            if let Some(image) = images.get(handle) {
                return Some(image);
            }
        }
    }
    None
}

fn create_fallback_image(template: &Image, color: [f32; 4]) -> Option<Image> {
    let format = template.texture_descriptor.format;
    let pixel = color_to_bytes(format, color)?;
    let mut image = Image::new_fill(
        template.texture_descriptor.size,
        TextureDimension::D2,
        &pixel,
        format,
        RenderAssetUsages::default(),
    );
    image.texture_view_descriptor = template.texture_view_descriptor.clone();
    Some(image)
}

fn color_to_bytes(format: TextureFormat, color: [f32; 4]) -> Option<Vec<u8>> {
    match format {
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => {
            let bytes: [u8; 4] = color.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8);
            Some(bytes.to_vec())
        }

        TextureFormat::R8Unorm => {
            let byte = (color[0].clamp(0.0, 1.0) * 255.0).round() as u8;
            Some(vec![byte])
        }

        TextureFormat::Rgba32Float => {
            let mut data = Vec::with_capacity(16);
            for component in color {
                data.extend_from_slice(&component.to_le_bytes());
            }
            Some(data)
        }
        _ => None,
    }
}
