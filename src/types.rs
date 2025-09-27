use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug, Encode, Decode)]
pub enum TileKind { Floor, Ramp }

#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode)]
pub struct Tile {
    pub kind: TileKind,
    pub tile_type: TileType,
    pub x: u32,
    pub y: u32,
    pub elevation: i8,   // can be negative for underwater, or positive for cliffs
}

#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode)]
pub enum TileType {
    Grass,
    Dirt,
    Cliff,
    Water,
}

#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub struct TileMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>, // row-major
}

impl TileMap {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            width: w, height: h,
            tiles: vec![Tile { kind: TileKind::Floor, tile_type: TileType::Grass, elevation: 0, x: 0, y: 0}; (w*h) as usize],
        }
    }
    pub fn idx(&self, x:u32,y:u32)->usize { (y*self.width + x) as usize }
    pub fn get(&self, x:u32,y:u32)->&Tile { &self.tiles[self.idx(x,y)] }
    pub fn set(&mut self, x:u32,y:u32, t:Tile){
        let i = self.idx(x, y);
        self.tiles[i] = t;
    }
}

pub const TILE_SIZE: f32 = 1.0;     // world units per tile
pub const TILE_HEIGHT: f32 = 1.0;   // height per elevation step
