use std::collections::HashMap;

use crate::types::{RampDirection, TILE_HEIGHT, TILE_SIZE, TileKind, TileMap, TileType};
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;

pub const CORNER_NW: usize = 0;
pub const CORNER_NE: usize = 1;
pub const CORNER_SW: usize = 2;
pub const CORNER_SE: usize = 3;

pub fn tile_corner_heights(map: &TileMap, x: u32, y: u32) -> [f32; 4] {
    let tile = map.get(x, y);
    let base = tile.elevation as f32 * TILE_HEIGHT;
    let mut corners = [base; 4];

    if tile.kind == TileKind::Ramp {
        let mut target = tile
            .ramp_direction
            .and_then(|dir| ramp_neighbor_height(map, x, y, dir, base).map(|h| (dir, h)));

        if target.is_none() {
            target = find_ramp_target(map, x, y, base);
        }

        if let Some((dir, neighbor_height)) = target {
            match dir {
                RampDirection::North => {
                    corners[CORNER_NW] = neighbor_height;
                    corners[CORNER_NE] = neighbor_height;
                }
                RampDirection::South => {
                    corners[CORNER_SW] = neighbor_height;
                    corners[CORNER_SE] = neighbor_height;
                }
                RampDirection::West => {
                    corners[CORNER_NW] = neighbor_height;
                    corners[CORNER_SW] = neighbor_height;
                }
                RampDirection::East => {
                    corners[CORNER_NE] = neighbor_height;
                    corners[CORNER_SE] = neighbor_height;
                }
            }
        }
    }

    corners
}

pub fn empty_mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
}

pub fn build_map_meshes(map: &TileMap) -> HashMap<TileType, Mesh> {
    let mut result = HashMap::new();

    if map.width == 0 || map.height == 0 {
        return result;
    }

    let mut buffers: HashMap<TileType, MeshBuffers> = HashMap::new();
    populate_mesh_buffers(map, Some(&mut buffers), None);

    for (tile_type, buffer) in buffers {
        result.insert(tile_type, buffer.into_mesh());
    }

    result
}

pub fn build_combined_mesh(map: &TileMap) -> Mesh {
    if map.width == 0 || map.height == 0 {
        return empty_mesh();
    }

    let mut combined = MeshBuffers::new(true);
    populate_mesh_buffers(map, None, Some(&mut combined));
    combined.into_mesh()
}

fn find_ramp_target(map: &TileMap, x: u32, y: u32, base: f32) -> Option<(RampDirection, f32)> {
    let mut result: Option<(RampDirection, f32)> = None;
    for dir in RampDirection::ALL {
        if let Some(height) = ramp_neighbor_height(map, x, y, dir, base) {
            match &result {
                Some((_, existing)) if *existing <= height => {}
                _ => {
                    result = Some((dir, height));
                }
            }
        }
    }
    result
}

fn ramp_neighbor_height(
    map: &TileMap,
    x: u32,
    y: u32,
    dir: RampDirection,
    base: f32,
) -> Option<f32> {
    let (dx, dy) = dir.offset();
    let nx = x as i32 + dx;
    let ny = y as i32 + dy;
    if nx < 0 || ny < 0 {
        return None;
    }
    let (ux, uy) = (nx as u32, ny as u32);
    if ux >= map.width || uy >= map.height {
        return None;
    }
    let neighbor = map.get(ux, uy);
    let height = neighbor.elevation as f32 * TILE_HEIGHT;
    if height < base { Some(height) } else { None }
}

fn build_corner_cache(map: &TileMap) -> Vec<[f32; 4]> {
    let mut cache = vec![[0.0f32; 4]; (map.width * map.height) as usize];
    for y in 0..map.height {
        for x in 0..map.width {
            let idx = map.idx(x, y);
            cache[idx] = tile_corner_heights(map, x, y);
        }
    }
    cache
}

fn populate_mesh_buffers(
    map: &TileMap,
    mut per_type: Option<&mut HashMap<TileType, MeshBuffers>>,
    mut combined: Option<&mut MeshBuffers>,
) {
    let corner_cache = build_corner_cache(map);

    for y in 0..map.height {
        for x in 0..map.width {
            let idx = map.idx(x, y);
            let tile = map.get(x, y);
            let corners = corner_cache[idx];
            let x0 = x as f32 * TILE_SIZE;
            let x1 = x0 + TILE_SIZE;
            let z0 = y as f32 * TILE_SIZE;
            let z1 = z0 + TILE_SIZE;

            let nw = Vec3::new(x0, corners[CORNER_NW], z0);
            let ne = Vec3::new(x1, corners[CORNER_NE], z0);
            let sw = Vec3::new(x0, corners[CORNER_SW], z1);
            let se = Vec3::new(x1, corners[CORNER_SE], z1);

            let north_neighbor = if y > 0 {
                corner_cache[map.idx(x, y - 1)]
            } else {
                [0.0; 4]
            };
            let south_neighbor = if y + 1 < map.height {
                corner_cache[map.idx(x, y + 1)]
            } else {
                [0.0; 4]
            };
            let west_neighbor = if x > 0 {
                corner_cache[map.idx(x - 1, y)]
            } else {
                [0.0; 4]
            };
            let east_neighbor = if x + 1 < map.width {
                corner_cache[map.idx(x + 1, y)]
            } else {
                [0.0; 4]
            };

            let north_bottom_a = Vec3::new(x0, north_neighbor[CORNER_SW].min(nw.y), z0);
            let north_bottom_b = Vec3::new(x1, north_neighbor[CORNER_SE].min(ne.y), z0);
            let south_bottom_a = Vec3::new(x1, south_neighbor[CORNER_NE].min(se.y), z1);
            let south_bottom_b = Vec3::new(x0, south_neighbor[CORNER_NW].min(sw.y), z1);
            let west_bottom_a = Vec3::new(x0, west_neighbor[CORNER_SE].min(sw.y), z1);
            let west_bottom_b = Vec3::new(x0, west_neighbor[CORNER_NE].min(nw.y), z0);
            let east_bottom_a = Vec3::new(x1, east_neighbor[CORNER_NW].min(ne.y), z0);
            let east_bottom_b = Vec3::new(x1, east_neighbor[CORNER_SW].min(se.y), z1);

            if let Some(buffers) = per_type.as_mut() {
                let buffer = buffers
                    .entry(tile.tile_type)
                    .or_insert_with(|| MeshBuffers::new(false));
                buffer.push_quad([nw, sw, se, ne], [[0.0, 0.0]; 4], None);
                buffer.add_side_face(
                    nw,
                    ne,
                    north_bottom_a,
                    north_bottom_b,
                    RampDirection::North,
                    None,
                );
                buffer.add_side_face(
                    se,
                    sw,
                    south_bottom_a,
                    south_bottom_b,
                    RampDirection::South,
                    None,
                );
                buffer.add_side_face(
                    sw,
                    nw,
                    west_bottom_a,
                    west_bottom_b,
                    RampDirection::West,
                    None,
                );
                buffer.add_side_face(
                    ne,
                    se,
                    east_bottom_a,
                    east_bottom_b,
                    RampDirection::East,
                    None,
                );
            }

            if let Some(combined_buffer) = combined.as_mut() {
                let tile_value = [tile.tile_type.as_index() as f32, 0.0];
                combined_buffer.push_quad([nw, sw, se, ne], [[0.0, 0.0]; 4], Some(tile_value));
                combined_buffer.add_side_face(
                    nw,
                    ne,
                    north_bottom_a,
                    north_bottom_b,
                    RampDirection::North,
                    Some(tile_value),
                );
                combined_buffer.add_side_face(
                    se,
                    sw,
                    south_bottom_a,
                    south_bottom_b,
                    RampDirection::South,
                    Some(tile_value),
                );
                combined_buffer.add_side_face(
                    sw,
                    nw,
                    west_bottom_a,
                    west_bottom_b,
                    RampDirection::West,
                    Some(tile_value),
                );
                combined_buffer.add_side_face(
                    ne,
                    se,
                    east_bottom_a,
                    east_bottom_b,
                    RampDirection::East,
                    Some(tile_value),
                );
            }
        }
    }
}

#[derive(Default)]
struct MeshBuffers {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    tile_types: Vec<[f32; 2]>,
    indices: Vec<u32>,
    next_index: u32,
    record_tile_types: bool,
}

impl MeshBuffers {
    fn new(record_tile_types: bool) -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            tile_types: Vec::new(),
            indices: Vec::new(),
            next_index: 0,
            record_tile_types,
        }
    }

    fn push_quad(&mut self, verts: [Vec3; 4], tex: [[f32; 2]; 4], tile: Option<[f32; 2]>) {
        self.push_triangle(verts[0], verts[1], verts[2], tex[0], tex[1], tex[2], tile);
        self.push_triangle(verts[0], verts[2], verts[3], tex[0], tex[2], tex[3], tile);
    }

    fn push_triangle(
        &mut self,
        a: Vec3,
        b: Vec3,
        c: Vec3,
        ta: [f32; 2],
        tb: [f32; 2],
        tc: [f32; 2],
        tile: Option<[f32; 2]>,
    ) {
        let normal = (b - a).cross(c - a).normalize_or_zero();
        self.positions.push(a.to_array());
        self.positions.push(b.to_array());
        self.positions.push(c.to_array());
        self.normals.push(normal.to_array());
        self.normals.push(normal.to_array());
        self.normals.push(normal.to_array());
        self.uvs.push(ta);
        self.uvs.push(tb);
        self.uvs.push(tc);

        if self.record_tile_types {
            let info = tile.unwrap_or([0.0, 0.0]);
            self.tile_types.push(info);
            self.tile_types.push(info);
            self.tile_types.push(info);
        }

        self.indices.extend_from_slice(&[
            self.next_index,
            self.next_index + 1,
            self.next_index + 2,
        ]);
        self.next_index += 3;
    }

    fn add_side_face(
        &mut self,
        top_a: Vec3,
        top_b: Vec3,
        bottom_a: Vec3,
        bottom_b: Vec3,
        direction: RampDirection,
        tile: Option<[f32; 2]>,
    ) {
        const EPS: f32 = 1e-4;
        if (top_a.y - bottom_a.y).abs() < EPS && (top_b.y - bottom_b.y).abs() < EPS {
            return;
        }

        let verts = [top_a, top_b, bottom_b, bottom_a];
        let tex = match direction {
            RampDirection::North => [[0.0, 0.0]; 4],
            RampDirection::South => [[0.0, 0.0]; 4],
            RampDirection::West => [[0.0, 0.0]; 4],
            RampDirection::East => [[0.0, 0.0]; 4],
        };

        self.push_quad(verts, tex, tile);
    }

    fn into_mesh(self) -> Mesh {
        let mut mesh = empty_mesh();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        if self.record_tile_types && !self.tile_types.is_empty() {
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, self.tile_types);
        }
        if !self.indices.is_empty() {
            mesh.insert_indices(Indices::U32(self.indices));
        }
        mesh
    }
}
