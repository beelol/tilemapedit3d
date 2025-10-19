use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainTextureMetadataEntry {
    pub id: String,
    pub diffuse: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness: Option<String>,
    pub splatmap_channel: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainWallTextureMetadata {
    pub id: String,
    pub diffuse: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainMetadata {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub tile_size: f32,
    pub textures: Vec<TerrainTextureMetadataEntry>,
    pub splatmap: String,
    pub mesh: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tilemap: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wall_texture: Option<TerrainWallTextureMetadata>,
}
