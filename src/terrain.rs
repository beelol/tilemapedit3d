use crate::types::{RampDirection, TILE_HEIGHT, TILE_SIZE, TileKind, TileMap, TileType};
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{
    PrimitiveTopology, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::texture::{Image, ImageSampler};

pub const CORNER_NW: usize = 0;
pub const CORNER_NE: usize = 1;
pub const CORNER_SW: usize = 2;
pub const CORNER_SE: usize = 3;

pub struct TerrainMeshResult {
    pub mesh: Mesh,
    pub splatmap: Image,
    pub map_size: Vec2,
}

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

pub fn build_terrain_mesh(map: &TileMap, uv_scale: f32) -> Option<TerrainMeshResult> {
    if map.width == 0 || map.height == 0 {
        return None;
    }

    let mut corner_cache = vec![[0.0f32; 4]; (map.width * map.height) as usize];
    for y in 0..map.height {
        for x in 0..map.width {
            let idx = map.idx(x, y);
            corner_cache[idx] = tile_corner_heights(map, x, y);
        }
    }

    let mut buffers = MeshBuffers::default();
    let mut splatmap = vec![0u8; (map.width * map.height * 4) as usize];

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

            buffers.push_quad([nw, sw, se, ne], UvMode::Xz, uv_scale);

            let (bnw, bne) = if y > 0 {
                let neighbor = corner_cache[map.idx(x, y - 1)];
                (neighbor[CORNER_SW], neighbor[CORNER_SE])
            } else {
                (0.0, 0.0)
            };
            buffers.add_side_face(
                nw,
                ne,
                Vec3::new(x0, bnw.min(nw.y), z0),
                Vec3::new(x1, bne.min(ne.y), z0),
                RampDirection::North,
                uv_scale,
            );

            let (bsw, bse) = if y + 1 < map.height {
                let neighbor = corner_cache[map.idx(x, y + 1)];
                (neighbor[CORNER_NW], neighbor[CORNER_NE])
            } else {
                (0.0, 0.0)
            };
            buffers.add_side_face(
                se,
                sw,
                Vec3::new(x1, bse.min(se.y), z1),
                Vec3::new(x0, bsw.min(sw.y), z1),
                RampDirection::South,
                uv_scale,
            );

            let (bnw, bsw) = if x > 0 {
                let neighbor = corner_cache[map.idx(x - 1, y)];
                (neighbor[CORNER_NE], neighbor[CORNER_SE])
            } else {
                (0.0, 0.0)
            };
            buffers.add_side_face(
                sw,
                nw,
                Vec3::new(x0, bsw.min(sw.y), z1),
                Vec3::new(x0, bnw.min(nw.y), z0),
                RampDirection::West,
                uv_scale,
            );

            let (bne, bse) = if x + 1 < map.width {
                let neighbor = corner_cache[map.idx(x + 1, y)];
                (neighbor[CORNER_NW], neighbor[CORNER_SW])
            } else {
                (0.0, 0.0)
            };
            buffers.add_side_face(
                ne,
                se,
                Vec3::new(x1, bne.min(ne.y), z0),
                Vec3::new(x1, bse.min(se.y), z1),
                RampDirection::East,
                uv_scale,
            );

            let pixel_index = ((y * map.width + x) * 4) as usize;
            let channel = tile_type_channel(tile.tile_type);
            splatmap[pixel_index + channel] = 255;
        }
    }

    let mesh = buffers.into_mesh();

    let mut image = Image::new(
        Extent3d {
            width: map.width,
            height: map.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        splatmap,
        TextureFormat::Rgba8Unorm,
    );
    image.sampler = ImageSampler::linear();
    image.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;

    let map_size = Vec2::new(map.width as f32 * TILE_SIZE, map.height as f32 * TILE_SIZE);

    Some(TerrainMeshResult {
        mesh,
        splatmap: image,
        map_size,
    })
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

#[derive(Default)]
struct MeshBuffers {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
    next_index: u32,
}

#[derive(Clone, Copy)]
enum UvMode {
    Xz,
    Xy,
    Zy,
}

fn tile_type_channel(tile_type: TileType) -> usize {
    match tile_type {
        TileType::Grass => 0,
        TileType::Dirt => 1,
        TileType::Cliff => 2,
        TileType::Water => 3,
    }
}

fn safe_scale(scale: f32) -> f32 {
    if scale.abs() < f32::EPSILON {
        1.0
    } else {
        scale
    }
}

fn uv_from_mode(pos: Vec3, mode: UvMode, scale: f32) -> [f32; 2] {
    let scale = safe_scale(scale);
    match mode {
        UvMode::Xz => [pos.x / scale, pos.z / scale],
        UvMode::Xy => [pos.x / scale, pos.y / scale],
        UvMode::Zy => [pos.z / scale, pos.y / scale],
    }
}

impl MeshBuffers {
    fn push_quad(&mut self, verts: [Vec3; 4], mode: UvMode, uv_scale: f32) {
        push_quad(
            &mut self.positions,
            &mut self.normals,
            &mut self.uvs,
            &mut self.indices,
            &mut self.next_index,
            verts,
            mode,
            uv_scale,
        );
    }

    fn add_side_face(
        &mut self,
        top_a: Vec3,
        top_b: Vec3,
        bottom_a: Vec3,
        bottom_b: Vec3,
        direction: RampDirection,
        uv_scale: f32,
    ) {
        add_side_face(
            &mut self.positions,
            &mut self.normals,
            &mut self.uvs,
            &mut self.indices,
            &mut self.next_index,
            top_a,
            top_b,
            bottom_a,
            bottom_b,
            direction,
            uv_scale,
        );
    }

    fn into_mesh(self) -> Mesh {
        let mut mesh = empty_mesh();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        if !self.indices.is_empty() {
            mesh.insert_indices(Indices::U32(self.indices));
        }
        mesh
    }
}

fn push_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    next_index: &mut u32,
    verts: [Vec3; 4],
    mode: UvMode,
    uv_scale: f32,
) {
    push_triangle(
        positions, normals, uvs, indices, next_index, verts[0], verts[1], verts[2], mode, uv_scale,
    );
    push_triangle(
        positions, normals, uvs, indices, next_index, verts[0], verts[2], verts[3], mode, uv_scale,
    );
}

fn push_triangle(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    next_index: &mut u32,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    mode: UvMode,
    uv_scale: f32,
) {
    let normal = (b - a).cross(c - a).normalize_or_zero();
    positions.push(a.to_array());
    positions.push(b.to_array());
    positions.push(c.to_array());
    normals.push(normal.to_array());
    normals.push(normal.to_array());
    normals.push(normal.to_array());
    uvs.push(uv_from_mode(a, mode, uv_scale));
    uvs.push(uv_from_mode(b, mode, uv_scale));
    uvs.push(uv_from_mode(c, mode, uv_scale));
    indices.extend_from_slice(&[*next_index, *next_index + 1, *next_index + 2]);
    *next_index += 3;
}

fn add_side_face(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    next_index: &mut u32,
    top_a: Vec3,
    top_b: Vec3,
    bottom_a: Vec3,
    bottom_b: Vec3,
    direction: RampDirection,
    uv_scale: f32,
) {
    const EPS: f32 = 1e-4;
    if (top_a.y - bottom_a.y).abs() < EPS && (top_b.y - bottom_b.y).abs() < EPS {
        return;
    }

    let (verts, mode) = match direction {
        RampDirection::North => ([top_a, top_b, bottom_b, bottom_a], UvMode::Xy),
        RampDirection::South => ([top_a, top_b, bottom_b, bottom_a], UvMode::Xy),
        RampDirection::West => ([top_a, top_b, bottom_b, bottom_a], UvMode::Zy),
        RampDirection::East => ([top_a, top_b, bottom_b, bottom_a], UvMode::Zy),
    };

    push_quad(
        positions, normals, uvs, indices, next_index, verts, mode, uv_scale,
    );
}
