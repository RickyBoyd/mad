use bevy::asset::RenderAssetUsages;
use bevy::color::ColorToComponents;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use crate::planet::generation::Tile;
use crate::planet::noise::{TerrainSample, sample_fractal_noise, sample_terrain};
use crate::planet::random::random_f32;
use crate::planet::resources::TerrainSettings;

const DETAIL_NOISE_OCTAVES: usize = 3;
const DETAIL_NOISE_LACUNARITY: f32 = 2.1;
const DETAIL_NOISE_PERSISTENCE: f32 = 0.5;

#[derive(Debug, Clone, Copy)]
struct SurfaceVertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
}

pub fn tile_field_to_mesh(
    tiles: &[Tile],
    plate_ids: &[usize],
    terrain_settings: TerrainSettings,
    seed: u64,
    lift: f32,
) -> Mesh {
    let patch_resolution = terrain_settings.surface_patch_resolution.max(1);
    let patch_count: usize = tiles.iter().map(|tile| tile.corners.len()).sum();
    let vertices_per_patch = (patch_resolution + 1) * (patch_resolution + 2) / 2;
    let indices_per_patch = patch_resolution * patch_resolution * 3;
    let total_vertices = patch_count * vertices_per_patch;
    let total_indices = patch_count * indices_per_patch;
    let noise_seed = seed ^ 0x52F6_5A2D_9C3B_6E17;

    let mut positions = Vec::<[f32; 3]>::with_capacity(total_vertices);
    let mut normals = Vec::<[f32; 3]>::with_capacity(total_vertices);
    let mut colors = Vec::<[f32; 4]>::with_capacity(total_vertices);
    let mut indices = Vec::<u32>::with_capacity(total_indices);

    for (tile_index, tile) in tiles.iter().enumerate() {
        append_tile_patch_to_mesh(
            tile,
            plate_ids[tile_index],
            terrain_settings,
            seed,
            noise_seed,
            lift,
            patch_resolution,
            &mut positions,
            &mut normals,
            &mut colors,
            &mut indices,
        );
    }

    build_mesh(positions, normals, colors, indices)
}

fn build_mesh(
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn append_tile_patch_to_mesh(
    tile: &Tile,
    plate_id: usize,
    terrain_settings: TerrainSettings,
    seed: u64,
    noise_seed: u64,
    lift: f32,
    patch_resolution: usize,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
) {
    let land_color = plate_color(seed, plate_id, tile.is_pentagon);

    for edge_index in 0..tile.corners.len() {
        let edge_start = tile.corners[edge_index];
        let edge_end = tile.corners[(edge_index + 1) % tile.corners.len()];

        append_patch_triangle(
            tile.center,
            edge_start,
            edge_end,
            land_color,
            terrain_settings,
            noise_seed,
            lift,
            patch_resolution,
            positions,
            normals,
            colors,
            indices,
        );
    }
}

fn append_patch_triangle(
    apex: Vec3,
    edge_start: Vec3,
    edge_end: Vec3,
    land_color: Color,
    terrain_settings: TerrainSettings,
    noise_seed: u64,
    lift: f32,
    patch_resolution: usize,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
) {
    let base_index = positions.len() as u32;
    let base_radius = apex.length();
    let apex_direction = apex.normalize_or_zero();
    let edge_start_direction = edge_start.normalize_or_zero();
    let edge_end_direction = edge_end.normalize_or_zero();
    let mut row_starts = Vec::with_capacity(patch_resolution + 1);

    for row in 0..=patch_resolution {
        row_starts.push(positions.len() as u32 - base_index);

        if row == 0 {
            push_sampled_vertex(
                apex_direction,
                base_radius,
                land_color,
                terrain_settings,
                noise_seed,
                lift,
                positions,
                normals,
                colors,
            );
            continue;
        }

        let row_t = row as f32 / patch_resolution as f32;
        let left = slerp_direction(apex_direction, edge_start_direction, row_t);
        let right = slerp_direction(apex_direction, edge_end_direction, row_t);

        for column in 0..=row {
            let column_t = column as f32 / row as f32;
            let direction = if column == 0 {
                left
            } else if column == row {
                right
            } else {
                slerp_direction(left, right, column_t)
            };

            push_sampled_vertex(
                direction,
                base_radius,
                land_color,
                terrain_settings,
                noise_seed,
                lift,
                positions,
                normals,
                colors,
            );
        }
    }

    for row in 0..patch_resolution {
        let current_row_start = base_index + row_starts[row];
        let next_row_start = base_index + row_starts[row + 1];

        for column in 0..=row {
            let top = current_row_start + column as u32;
            let bottom_left = next_row_start + column as u32;
            let bottom_right = next_row_start + column as u32 + 1;

            append_oriented_triangle(top, bottom_left, bottom_right, positions, indices);

            if column < row {
                let top_right = current_row_start + column as u32 + 1;
                append_oriented_triangle(top, bottom_right, top_right, positions, indices);
            }
        }
    }
}

fn push_sampled_vertex(
    direction: Vec3,
    base_radius: f32,
    land_color: Color,
    terrain_settings: TerrainSettings,
    noise_seed: u64,
    lift: f32,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
) {
    let vertex = sample_surface_vertex(
        direction,
        base_radius,
        lift,
        land_color,
        terrain_settings,
        noise_seed,
    );
    positions.push(vertex.position);
    normals.push(vertex.normal);
    colors.push(vertex.color);
}

fn sample_surface_vertex(
    direction: Vec3,
    base_radius: f32,
    lift: f32,
    land_color: Color,
    terrain_settings: TerrainSettings,
    noise_seed: u64,
) -> SurfaceVertex {
    let direction = direction.normalize_or_zero();
    let sample = sample_terrain(direction, terrain_settings, noise_seed);
    let position = sample_surface_position(
        direction,
        sample,
        base_radius,
        lift,
        terrain_settings,
        noise_seed,
    );
    let normal =
        estimate_surface_normal(direction, base_radius, lift, terrain_settings, noise_seed);

    SurfaceVertex {
        position: position.to_array(),
        normal: normal.to_array(),
        color: surface_color(land_color, sample).to_linear().to_f32_array(),
    }
}

fn sample_surface_position(
    direction: Vec3,
    sample: TerrainSample,
    base_radius: f32,
    lift: f32,
    terrain_settings: TerrainSettings,
    noise_seed: u64,
) -> Vec3 {
    let direction = direction.normalize_or_zero();
    if sample.is_water {
        return direction * (base_radius + lift + sample.surface_height);
    }

    let warped_direction = perturb_surface_direction(direction, terrain_settings, noise_seed);
    let radial_offset = sample_fractal_noise(
        direction,
        terrain_settings.surface_detail_frequency * 1.37,
        DETAIL_NOISE_OCTAVES,
        DETAIL_NOISE_LACUNARITY,
        DETAIL_NOISE_PERSISTENCE,
        noise_seed ^ 0xD1B5_4A32_9C63_E7F1,
    ) * terrain_settings.surface_detail_radial_amplitude;

    warped_direction * (base_radius + lift + sample.surface_height + radial_offset)
}

fn estimate_surface_normal(
    direction: Vec3,
    base_radius: f32,
    lift: f32,
    terrain_settings: TerrainSettings,
    noise_seed: u64,
) -> Vec3 {
    let direction = direction.normalize_or_zero();
    let reference = if direction.y.abs() < 0.99 {
        Vec3::Y
    } else {
        Vec3::X
    };
    let tangent_x = direction.cross(reference).normalize_or_zero();
    let tangent_y = tangent_x.cross(direction).normalize_or_zero();
    let sample_angle = terrain_settings.normal_sample_angle.max(0.001);

    let x_plus = sample_surface_position(
        rotated_direction(direction, tangent_x, sample_angle),
        sample_terrain(
            rotated_direction(direction, tangent_x, sample_angle),
            terrain_settings,
            noise_seed,
        ),
        base_radius,
        lift,
        terrain_settings,
        noise_seed,
    );
    let x_minus = sample_surface_position(
        rotated_direction(direction, -tangent_x, sample_angle),
        sample_terrain(
            rotated_direction(direction, -tangent_x, sample_angle),
            terrain_settings,
            noise_seed,
        ),
        base_radius,
        lift,
        terrain_settings,
        noise_seed,
    );
    let y_plus = sample_surface_position(
        rotated_direction(direction, tangent_y, sample_angle),
        sample_terrain(
            rotated_direction(direction, tangent_y, sample_angle),
            terrain_settings,
            noise_seed,
        ),
        base_radius,
        lift,
        terrain_settings,
        noise_seed,
    );
    let y_minus = sample_surface_position(
        rotated_direction(direction, -tangent_y, sample_angle),
        sample_terrain(
            rotated_direction(direction, -tangent_y, sample_angle),
            terrain_settings,
            noise_seed,
        ),
        base_radius,
        lift,
        terrain_settings,
        noise_seed,
    );

    let normal = (x_plus - x_minus)
        .cross(y_plus - y_minus)
        .normalize_or_zero();
    if normal.length_squared() <= f32::EPSILON {
        direction
    } else if normal.dot(direction) < 0.0 {
        -normal
    } else {
        normal
    }
}

fn rotated_direction(direction: Vec3, tangent: Vec3, angle: f32) -> Vec3 {
    (direction * angle.cos() + tangent * angle.sin()).normalize_or_zero()
}

fn perturb_surface_direction(
    direction: Vec3,
    terrain_settings: TerrainSettings,
    noise_seed: u64,
) -> Vec3 {
    let amplitude = terrain_settings.surface_detail_tangent_amplitude.max(0.0);
    if amplitude <= f32::EPSILON {
        return direction;
    }

    let reference = if direction.y.abs() < 0.99 {
        Vec3::Y
    } else {
        Vec3::X
    };
    let tangent_x = direction.cross(reference).normalize_or_zero();
    let tangent_y = tangent_x.cross(direction).normalize_or_zero();
    let detail_frequency = terrain_settings.surface_detail_frequency.max(0.001);
    let noise_x = sample_fractal_noise(
        direction,
        detail_frequency,
        DETAIL_NOISE_OCTAVES,
        DETAIL_NOISE_LACUNARITY,
        DETAIL_NOISE_PERSISTENCE,
        noise_seed ^ 0x8F32_1B74_C6A5_9DE1,
    );
    let noise_y = sample_fractal_noise(
        direction,
        detail_frequency,
        DETAIL_NOISE_OCTAVES,
        DETAIL_NOISE_LACUNARITY,
        DETAIL_NOISE_PERSISTENCE,
        noise_seed ^ 0x51AF_D7C3_284E_B609,
    );

    (direction + tangent_x * noise_x * amplitude + tangent_y * noise_y * amplitude)
        .normalize_or_zero()
}

fn slerp_direction(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    let a = a.normalize_or_zero();
    let b = b.normalize_or_zero();
    let dot = a.dot(b).clamp(-1.0, 1.0);

    if dot > 0.9995 {
        return a.lerp(b, t).normalize_or_zero();
    }

    let theta = dot.acos();
    let sin_theta = theta.sin();

    if sin_theta.abs() <= f32::EPSILON {
        return a;
    }

    let a_weight = ((1.0 - t) * theta).sin() / sin_theta;
    let b_weight = (t * theta).sin() / sin_theta;
    (a * a_weight + b * b_weight).normalize_or_zero()
}

fn append_oriented_triangle(
    a: u32,
    b: u32,
    c: u32,
    positions: &[[f32; 3]],
    indices: &mut Vec<u32>,
) {
    let position_a = Vec3::from_array(positions[a as usize]);
    let position_b = Vec3::from_array(positions[b as usize]);
    let position_c = Vec3::from_array(positions[c as usize]);
    let face_normal = (position_b - position_a).cross(position_c - position_a);
    let outward_hint = position_a + position_b + position_c;

    if face_normal.dot(outward_hint) >= 0.0 {
        indices.extend_from_slice(&[a, b, c]);
    } else {
        indices.extend_from_slice(&[a, c, b]);
    }
}

fn surface_color(land_color: Color, sample: TerrainSample) -> Color {
    if sample.is_water {
        let depth = (sample.surface_height - sample.terrain_height).max(0.0);
        let depth_tint = (depth * 2.5).clamp(0.0, 1.0);
        Color::srgb(
            0.04 + depth_tint * 0.02,
            0.22 + depth_tint * 0.08,
            0.55 + depth_tint * 0.20,
        )
    } else {
        land_color
    }
}

fn plate_color(seed: u64, plate_id: usize, is_pentagon: bool) -> Color {
    let plate_seed = seed ^ (plate_id as u64 + 1).wrapping_mul(0xD6E8_FD9D_3F5A_BC31);
    let hue = random_f32(plate_seed) * 360.0;
    let saturation = 0.55 + random_f32(plate_seed ^ 0xA076_1D64_78BD_642F) * 0.25;
    let lightness = if is_pentagon {
        0.58 + random_f32(plate_seed ^ 0xE703_7ED1_A0B4_28DB) * 0.14
    } else {
        0.42 + random_f32(plate_seed ^ 0x8EBC_6AF0_9C88_C6E3) * 0.16
    };

    Color::hsl(hue, saturation.min(1.0), lightness.min(1.0))
}
