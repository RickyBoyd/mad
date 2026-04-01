use std::collections::HashMap;

use bevy::prelude::*;

use crate::planet::random::Random;
use crate::planet::resources::PlateGrowthSettings;

const MAX_TILE_NEIGHBORS: usize = 6;
const UNASSIGNED_PLATE: usize = usize::MAX;

#[derive(Debug, Clone)]
pub struct TriMesh {
    pub positions: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub center: Vec3,
    pub corners: Vec<Vec3>,
    pub is_pentagon: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TileNeighbors {
    count: u8,
    indices: [usize; MAX_TILE_NEIGHBORS],
}

impl TileNeighbors {
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.indices[..self.count as usize].iter().copied()
    }

    fn insert(&mut self, neighbor: usize) {
        if self.iter().any(|existing| existing == neighbor) {
            return;
        }

        let index = self.count as usize;
        debug_assert!(index < MAX_TILE_NEIGHBORS);
        self.indices[index] = neighbor;
        self.count += 1;
    }
}

pub fn build_icosphere(radius: f32, subdivisions: usize) -> TriMesh {
    let mut mesh = create_icosahedron(radius);
    for _ in 0..subdivisions {
        mesh = subdivide_once(&mesh, radius);
    }
    mesh
}

pub fn build_tiles(mesh: &TriMesh, radius: f32) -> Vec<Tile> {
    let centroids = triangle_centroids(mesh, radius);
    let incidents = incident_triangles(mesh);

    let mut tiles = Vec::with_capacity(mesh.positions.len());

    for (vertex_index, tri_list) in incidents.iter().enumerate() {
        let center = mesh.positions[vertex_index].normalize() * radius;

        let mut corners = tri_list
            .iter()
            .map(|&tri_idx| centroids[tri_idx])
            .collect::<Vec<_>>();

        sort_corners_around(center, &mut corners);

        tiles.push(Tile {
            center,
            is_pentagon: corners.len() == 5,
            corners,
        });
    }

    tiles
}

pub fn build_tile_neighbors(mesh: &TriMesh) -> Vec<TileNeighbors> {
    let mut neighbors = vec![TileNeighbors::default(); mesh.positions.len()];

    for [a, b, c] in &mesh.triangles {
        connect_tiles(&mut neighbors, *a as usize, *b as usize);
        connect_tiles(&mut neighbors, *b as usize, *c as usize);
        connect_tiles(&mut neighbors, *c as usize, *a as usize);
    }

    neighbors
}

pub fn build_plates(
    tiles: &[Tile],
    neighbors: &[TileNeighbors],
    plate_count: usize,
    growth: PlateGrowthSettings,
    seed: u64,
) -> Vec<usize> {
    let plate_count = plate_count.min(neighbors.len());
    debug_assert!(plate_count <= u16::BITS as usize);

    let mut rng = Random::new(seed ^ 0x6A09_E667_F3BC_C909);
    let mut plate_ids = vec![UNASSIGNED_PLATE; neighbors.len()];
    let mut plate_boundaries = vec![Vec::new(); plate_count];
    let mut boundary_membership = vec![0u16; neighbors.len()];
    let mut seed_tiles = Vec::with_capacity(plate_count);

    while seed_tiles.len() < plate_count {
        let tile_index = rng.index(neighbors.len());
        if seed_tiles.contains(&tile_index) {
            continue;
        }

        let plate_id = seed_tiles.len();
        seed_tiles.push(tile_index);
        plate_ids[tile_index] = plate_id;
        enqueue_plate_boundary(
            plate_id,
            tile_index,
            neighbors,
            &plate_ids,
            &mut plate_boundaries,
            &mut boundary_membership,
        );
    }

    let plate_seeds = seed_tiles
        .iter()
        .map(|&tile_index| tiles[tile_index].center.normalize())
        .collect::<Vec<_>>();

    let mut assigned_tiles = plate_count;

    while assigned_tiles < neighbors.len() {
        let mut expanded = false;

        for _ in 0..plate_count {
            let plate_id = rng.index(plate_count);
            let claimed = expand_plate_batch(
                plate_id,
                tiles,
                &plate_seeds,
                neighbors,
                &mut plate_ids,
                &mut plate_boundaries,
                &mut boundary_membership,
                growth,
                &mut rng,
            );
            if claimed > 0 {
                assigned_tiles += claimed;
                expanded = true;
                break;
            }
        }

        if expanded {
            continue;
        }

        for plate_id in 0..plate_count {
            let claimed = expand_plate_batch(
                plate_id,
                tiles,
                &plate_seeds,
                neighbors,
                &mut plate_ids,
                &mut plate_boundaries,
                &mut boundary_membership,
                growth,
                &mut rng,
            );
            if claimed > 0 {
                assigned_tiles += claimed;
                expanded = true;
                break;
            }
        }

        if !expanded {
            break;
        }
    }

    plate_ids
}

fn connect_tiles(neighbors: &mut [TileNeighbors], a: usize, b: usize) {
    neighbors[a].insert(b);
    neighbors[b].insert(a);
}

fn enqueue_plate_boundary(
    plate_id: usize,
    tile_index: usize,
    neighbors: &[TileNeighbors],
    plate_ids: &[usize],
    plate_boundaries: &mut [Vec<usize>],
    boundary_membership: &mut [u16],
) {
    let plate_mask = 1u16 << plate_id;

    for neighbor in neighbors[tile_index].iter() {
        if plate_ids[neighbor] != UNASSIGNED_PLATE {
            continue;
        }
        if boundary_membership[neighbor] & plate_mask != 0 {
            continue;
        }

        boundary_membership[neighbor] |= plate_mask;
        plate_boundaries[plate_id].push(neighbor);
    }
}

fn expand_plate_batch(
    plate_id: usize,
    tiles: &[Tile],
    plate_seeds: &[Vec3],
    neighbors: &[TileNeighbors],
    plate_ids: &mut [usize],
    plate_boundaries: &mut [Vec<usize>],
    boundary_membership: &mut [u16],
    growth: PlateGrowthSettings,
    rng: &mut Random,
) -> usize {
    let plate_mask = 1u16 << plate_id;
    let boundary = &mut plate_boundaries[plate_id];
    let seed_direction = plate_seeds[plate_id];

    let mut best_index = None;
    let mut best_score = f32::NEG_INFINITY;
    let mut i = 0;

    while i < boundary.len() {
        let candidate = boundary[i];

        if plate_ids[candidate] != UNASSIGNED_PLATE {
            boundary.swap_remove(i);
            continue;
        }

        let score = boundary_candidate_score(
            plate_id,
            candidate,
            tiles,
            neighbors,
            plate_ids,
            seed_direction,
            rng,
        );

        if score > best_score {
            best_score = score;
            best_index = Some(i);
        }

        i += 1;
    }

    let Some(best_index) = best_index else {
        return 0;
    };

    let anchor_tile = boundary.swap_remove(best_index);
    let growth_direction = tiles[anchor_tile].center.normalize();
    let target_batch_size = growth.batch_min
        + if growth.batch_variation == 0 {
            0
        } else {
            rng.index(growth.batch_variation)
        };

    let mut staged_tiles = Vec::with_capacity(target_batch_size.max(1));
    staged_tiles.push(anchor_tile);

    while staged_tiles.len() < target_batch_size {
        let mut next_tile = None;
        let mut next_score = f32::NEG_INFINITY;

        for &staged_tile in &staged_tiles {
            for neighbor in neighbors[staged_tile].iter() {
                if plate_ids[neighbor] != UNASSIGNED_PLATE {
                    continue;
                }
                if staged_tiles.contains(&neighbor) {
                    continue;
                }

                let score = batch_candidate_score(
                    plate_id,
                    neighbor,
                    tiles,
                    neighbors,
                    plate_ids,
                    &staged_tiles,
                    seed_direction,
                    growth_direction,
                    rng,
                );

                if score > next_score {
                    next_score = score;
                    next_tile = Some(neighbor);
                }
            }
        }

        let Some(next_tile) = next_tile else {
            break;
        };

        staged_tiles.push(next_tile);
    }

    let claimed_tiles = staged_tiles.len();

    for &tile_index in &staged_tiles {
        boundary_membership[tile_index] &= !plate_mask;
        plate_ids[tile_index] = plate_id;
    }

    for &tile_index in &staged_tiles {
        enqueue_plate_boundary(
            plate_id,
            tile_index,
            neighbors,
            plate_ids,
            plate_boundaries,
            boundary_membership,
        );
    }

    claimed_tiles
}

fn boundary_candidate_score(
    plate_id: usize,
    tile_index: usize,
    tiles: &[Tile],
    neighbors: &[TileNeighbors],
    plate_ids: &[usize],
    seed_direction: Vec3,
    rng: &mut Random,
) -> f32 {
    let shared_border = neighbors[tile_index]
        .iter()
        .filter(|&neighbor| plate_ids[neighbor] == plate_id)
        .count() as f32;
    let candidate_direction = tiles[tile_index].center.normalize();
    let distance_penalty = 1.0 - candidate_direction.dot(seed_direction);
    let jitter = (rng.next_u64() as f64 / u64::MAX as f64) as f32 * 0.15;

    shared_border * 4.0 - distance_penalty * 1.5 + jitter
}

fn batch_candidate_score(
    plate_id: usize,
    tile_index: usize,
    tiles: &[Tile],
    neighbors: &[TileNeighbors],
    plate_ids: &[usize],
    staged_tiles: &[usize],
    seed_direction: Vec3,
    growth_direction: Vec3,
    rng: &mut Random,
) -> f32 {
    let support = neighbors[tile_index]
        .iter()
        .filter(|&neighbor| plate_ids[neighbor] == plate_id || staged_tiles.contains(&neighbor))
        .count() as f32;
    let candidate_direction = tiles[tile_index].center.normalize();
    let seed_distance_penalty = 1.0 - candidate_direction.dot(seed_direction);
    let direction_alignment = candidate_direction.dot(growth_direction);
    let jitter = (rng.next_u64() as f64 / u64::MAX as f64) as f32 * 0.1;
    let thin_penalty = if support < 2.0 { 2.5 } else { 0.0 };

    support * 5.0 + direction_alignment * 2.5 - seed_distance_penalty - thin_penalty + jitter
}

fn incident_triangles(mesh: &TriMesh) -> Vec<Vec<usize>> {
    let mut result = vec![Vec::new(); mesh.positions.len()];

    for (tri_idx, [a, b, c]) in mesh.triangles.iter().copied().enumerate() {
        result[a as usize].push(tri_idx);
        result[b as usize].push(tri_idx);
        result[c as usize].push(tri_idx);
    }

    result
}

fn triangle_centroids(mesh: &TriMesh, radius: f32) -> Vec<Vec3> {
    mesh.triangles
        .iter()
        .map(|[a, b, c]| {
            let p = (mesh.positions[*a as usize]
                + mesh.positions[*b as usize]
                + mesh.positions[*c as usize])
                / 3.0;

            p.normalize() * radius
        })
        .collect()
}

fn orthonormal_basis(normal: Vec3) -> (Vec3, Vec3) {
    let helper = if normal.y.abs() < 0.99 {
        Vec3::Y
    } else {
        Vec3::X
    };
    let tangent = normal.cross(helper).normalize();
    let bitangent = normal.cross(tangent).normalize();
    (tangent, bitangent)
}

fn sort_corners_around(center: Vec3, corners: &mut [Vec3]) {
    let normal = center.normalize();
    let (tangent, bitangent) = orthonormal_basis(normal);

    corners.sort_by(|a, b| {
        let va = (*a - center).normalize_or_zero();
        let vb = (*b - center).normalize_or_zero();

        let angle_a = va.dot(bitangent).atan2(va.dot(tangent));
        let angle_b = vb.dot(bitangent).atan2(vb.dot(tangent));

        angle_a.partial_cmp(&angle_b).unwrap()
    });
}

fn midpoint(
    a: u32,
    b: u32,
    positions: &mut Vec<Vec3>,
    cache: &mut HashMap<(u32, u32), u32>,
    radius: f32,
) -> u32 {
    let key = if a < b { (a, b) } else { (b, a) };

    if let Some(&idx) = cache.get(&key) {
        return idx;
    }

    let pa = positions[a as usize];
    let pb = positions[b as usize];
    let pm = ((pa + pb) * 0.5).normalize() * radius;

    let idx = positions.len() as u32;
    positions.push(pm);
    cache.insert(key, idx);
    idx
}

fn subdivide_once(mesh: &TriMesh, radius: f32) -> TriMesh {
    let mut positions = mesh.positions.clone();
    let mut triangles = Vec::with_capacity(mesh.triangles.len() * 4);
    let mut cache = HashMap::new();

    for [a, b, c] in &mesh.triangles {
        let ab = midpoint(*a, *b, &mut positions, &mut cache, radius);
        let bc = midpoint(*b, *c, &mut positions, &mut cache, radius);
        let ca = midpoint(*c, *a, &mut positions, &mut cache, radius);

        triangles.push([*a, ab, ca]);
        triangles.push([*b, bc, ab]);
        triangles.push([*c, ca, bc]);
        triangles.push([ab, bc, ca]);
    }

    TriMesh {
        positions,
        triangles,
    }
}

fn create_icosahedron(radius: f32) -> TriMesh {
    let phi = (1.0 + 5.0_f32.sqrt()) * 0.5;

    let mut positions = vec![
        Vec3::new(-1.0, phi, 0.0),
        Vec3::new(1.0, phi, 0.0),
        Vec3::new(-1.0, -phi, 0.0),
        Vec3::new(1.0, -phi, 0.0),
        Vec3::new(0.0, -1.0, phi),
        Vec3::new(0.0, 1.0, phi),
        Vec3::new(0.0, -1.0, -phi),
        Vec3::new(0.0, 1.0, -phi),
        Vec3::new(phi, 0.0, -1.0),
        Vec3::new(phi, 0.0, 1.0),
        Vec3::new(-phi, 0.0, -1.0),
        Vec3::new(-phi, 0.0, 1.0),
    ];

    for p in &mut positions {
        *p = p.normalize() * radius;
    }

    let triangles = vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];

    TriMesh {
        positions,
        triangles,
    }
}
