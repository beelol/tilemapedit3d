use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum TileKind {
    Floor,
    Ramp,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug, Default)]
pub enum Orientation4 {
    #[default]
    North,
    East,
    South,
    West,
}

impl Orientation4 {
    pub fn next(self) -> Self {
        match self {
            Orientation4::North => Orientation4::East,
            Orientation4::East => Orientation4::South,
            Orientation4::South => Orientation4::West,
            Orientation4::West => Orientation4::North,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tile {
    pub kind: TileKind,
    pub tile_type: TileType,
    pub x: u32,
    pub y: u32,
    pub elevation: i8, // can be negative for underwater, or positive for cliffs
    #[serde(default)]
    pub orientation: Orientation4,
    #[serde(default)]
    pub manual_orientation: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TileType {
    Grass,
    Dirt,
    Cliff,
    Water,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TileMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>, // row-major
}

impl TileMap {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            width: w,
            height: h,
            tiles: vec![
                Tile {
                    kind: TileKind::Floor,
                    tile_type: TileType::Grass,
                    elevation: 0,
                    x: 0,
                    y: 0,
                    orientation: Orientation4::North,
                    manual_orientation: false,
                };
                (w * h) as usize
            ],
        }
    }
    pub fn idx(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }
    pub fn get(&self, x: u32, y: u32) -> &Tile {
        &self.tiles[self.idx(x, y)]
    }
    pub fn set(&mut self, x: u32, y: u32, t: Tile) {
        let i = self.idx(x, y);
        self.tiles[i] = t;
    }
}

pub const TILE_SIZE: f32 = 1.0; // world units per tile
pub const TILE_HEIGHT: f32 = 1.0; // height per elevation step
