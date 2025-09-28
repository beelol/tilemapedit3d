use crate::types::{Orientation4, TILE_HEIGHT, TILE_SIZE, TileKind, TileMap};
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl From<Orientation4> for Direction {
    fn from(value: Orientation4) -> Self {
        match value {
            Orientation4::North => Direction::North,
            Orientation4::East => Direction::East,
            Orientation4::South => Direction::South,
            Orientation4::West => Direction::West,
        }
    }
}

const CORNER_NW: usize = 0;
const CORNER_NE: usize = 1;
const CORNER_SW: usize = 2;
const CORNER_SE: usize = 3;

pub fn build_map_mesh(map: &TileMap) -> Mesh {
    if map.width == 0 || map.height == 0 {
        return Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
    }

    // Cache corner heights for every tile so we can stitch seams reliably.
    let mut corner_cache = vec![[0.0f32; 4]; (map.width * map.height) as usize];
    for y in 0..map.height {
        for x in 0..map.width {
            let idx = map.idx(x, y);
            let tile = map.get(x, y);
            let base = tile.elevation as f32 * TILE_HEIGHT;
            let mut corners = [base; 4];

            if tile.kind == TileKind::Ramp {
                if let Some((dir, neighbor_height)) =
                    find_ramp_target(map, x, y, base, tile.orientation)
                {
                    match dir {
                        Direction::North => {
                            corners[CORNER_NW] = neighbor_height;
                            corners[CORNER_NE] = neighbor_height;
                        }
                        Direction::South => {
                            corners[CORNER_SW] = neighbor_height;
                            corners[CORNER_SE] = neighbor_height;
                        }
                        Direction::West => {
                            corners[CORNER_NW] = neighbor_height;
                            corners[CORNER_SW] = neighbor_height;
                        }
                        Direction::East => {
                            corners[CORNER_NE] = neighbor_height;
                            corners[CORNER_SE] = neighbor_height;
                        }
                    }
                }
            }

            corner_cache[idx] = corners;
        }
    }

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let mut next_index: u32 = 0;

    for y in 0..map.height {
        for x in 0..map.width {
            let idx = map.idx(x, y);
            let corners = corner_cache[idx];
            let x0 = x as f32 * TILE_SIZE;
            let x1 = x0 + TILE_SIZE;
            let z0 = y as f32 * TILE_SIZE;
            let z1 = z0 + TILE_SIZE;

            let nw = Vec3::new(x0, corners[CORNER_NW], z0);
            let ne = Vec3::new(x1, corners[CORNER_NE], z0);
            let sw = Vec3::new(x0, corners[CORNER_SW], z1);
            let se = Vec3::new(x1, corners[CORNER_SE], z1);

            push_quad(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                &mut next_index,
                [nw, sw, se, ne],
                [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
            );

            // North edge (towards y-1)
            let (bnw, bne) = if y > 0 {
                let neighbor = corner_cache[map.idx(x, y - 1)];
                (neighbor[CORNER_SW], neighbor[CORNER_SE])
            } else {
                (0.0, 0.0)
            };
            add_side_face(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                &mut next_index,
                nw,
                ne,
                Vec3::new(x0, bnw.min(nw.y), z0),
                Vec3::new(x1, bne.min(ne.y), z0),
                Direction::North,
            );

            // South edge (towards y+1)
            let (bsw, bse) = if y + 1 < map.height {
                let neighbor = corner_cache[map.idx(x, y + 1)];
                (neighbor[CORNER_NW], neighbor[CORNER_NE])
            } else {
                (0.0, 0.0)
            };
            add_side_face(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                &mut next_index,
                se,
                sw,
                Vec3::new(x1, bse.min(se.y), z1),
                Vec3::new(x0, bsw.min(sw.y), z1),
                Direction::South,
            );

            // West edge (towards x-1)
            let (bnw, bsw) = if x > 0 {
                let neighbor = corner_cache[map.idx(x - 1, y)];
                (neighbor[CORNER_NE], neighbor[CORNER_SE])
            } else {
                (0.0, 0.0)
            };
            add_side_face(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                &mut next_index,
                sw,
                nw,
                Vec3::new(x0, bsw.min(sw.y), z1),
                Vec3::new(x0, bnw.min(nw.y), z0),
                Direction::West,
            );

            // East edge (towards x+1)
            let (bne, bse) = if x + 1 < map.width {
                let neighbor = corner_cache[map.idx(x + 1, y)];
                (neighbor[CORNER_NW], neighbor[CORNER_SW])
            } else {
                (0.0, 0.0)
            };
            add_side_face(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                &mut next_index,
                ne,
                se,
                Vec3::new(x1, bne.min(ne.y), z0),
                Vec3::new(x1, bse.min(se.y), z1),
                Direction::East,
            );
        }
    }

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn neighbor_offset(dir: Direction) -> (i32, i32) {
    match dir {
        Direction::North => (0, -1),
        Direction::East => (1, 0),
        Direction::South => (0, 1),
        Direction::West => (-1, 0),
    }
}

fn find_ramp_target(
    map: &TileMap,
    x: u32,
    y: u32,
    base: f32,
    orientation: Orientation4,
) -> Option<(Direction, f32)> {
    const EPS: f32 = 1e-4;

    let mut consider = |dir: Direction| -> Option<(Direction, f32, f32)> {
        let (dx, dy) = neighbor_offset(dir);
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
        let h = neighbor.elevation as f32 * TILE_HEIGHT;
        if h > base {
            let diff = h - base;
            Some((dir, h, diff))
        } else {
            None
        }
    };

    let preferred: Direction = orientation.into();
    if let Some((dir, height, _)) = consider(preferred) {
        return Some((dir, height));
    }

    let mut best: Option<(Direction, f32, f32)> = None;
    for dir in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        if dir == preferred {
            continue;
        }
        if let Some(candidate) = consider(dir) {
            match best {
                None => best = Some(candidate),
                Some((_, _, best_diff)) => {
                    if candidate.2 + EPS < best_diff {
                        best = Some(candidate);
                    }
                }
            }
        }
    }

    best.map(|(dir, height, _)| (dir, height))
}

fn push_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    next_index: &mut u32,
    verts: [Vec3; 4],
    tex: [[f32; 2]; 4],
) {
    push_triangle(
        positions, normals, uvs, indices, next_index, verts[0], verts[1], verts[2], tex[0], tex[1],
        tex[2],
    );
    push_triangle(
        positions, normals, uvs, indices, next_index, verts[0], verts[2], verts[3], tex[0], tex[2],
        tex[3],
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
    ta: [f32; 2],
    tb: [f32; 2],
    tc: [f32; 2],
) {
    let normal = (b - a).cross(c - a).normalize_or_zero();
    positions.push(a.to_array());
    positions.push(b.to_array());
    positions.push(c.to_array());
    normals.push(normal.to_array());
    normals.push(normal.to_array());
    normals.push(normal.to_array());
    uvs.push(ta);
    uvs.push(tb);
    uvs.push(tc);
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
    direction: Direction,
) {
    const EPS: f32 = 1e-4;
    if (top_a.y - bottom_a.y).abs() < EPS && (top_b.y - bottom_b.y).abs() < EPS {
        return;
    }

    let (verts, tex) = match direction {
        Direction::North => (
            [top_a, top_b, bottom_b, bottom_a],
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        ),
        Direction::South => (
            [top_a, top_b, bottom_b, bottom_a],
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        ),
        Direction::West => (
            [top_a, top_b, bottom_b, bottom_a],
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        ),
        Direction::East => (
            [top_a, top_b, bottom_b, bottom_a],
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        ),
    };

    push_quad(positions, normals, uvs, indices, next_index, verts, tex);
}
