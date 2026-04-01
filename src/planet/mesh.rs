use bevy::asset::RenderAssetUsages;
use bevy::color::ColorToComponents;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use crate::planet::generation::Tile;
use crate::planet::random::random_f32;

pub fn tile_field_to_mesh(tiles: &[Tile], plate_ids: &[usize], seed: u64, lift: f32) -> Mesh {
    let total_vertices: usize = tiles.iter().map(|tile| tile.corners.len() + 1).sum();
    let total_indices: usize = tiles.iter().map(|tile| tile.corners.len() * 3).sum();

    let mut positions = Vec::<[f32; 3]>::with_capacity(total_vertices);
    let mut normals = Vec::<[f32; 3]>::with_capacity(total_vertices);
    let mut colors = Vec::<[f32; 4]>::with_capacity(total_vertices);
    let mut indices = Vec::<u32>::with_capacity(total_indices);

    for (tile_index, tile) in tiles.iter().enumerate() {
        let color = plate_color(seed, plate_ids[tile_index], tile.is_pentagon);
        append_tile_to_mesh(
            tile,
            color,
            lift,
            &mut positions,
            &mut normals,
            &mut colors,
            &mut indices,
        );
    }

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

fn append_tile_to_mesh(
    tile: &Tile,
    color: Color,
    lift: f32,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
) {
    let vertex_offset = positions.len() as u32;
    let center = tile.center.normalize() * (tile.center.length() + lift);
    let color = color.to_linear().to_f32_array();

    positions.push(center.to_array());
    normals.push(center.normalize().to_array());
    colors.push(color);

    for corner in &tile.corners {
        let p = corner.normalize() * (corner.length() + lift);
        positions.push(p.to_array());
        normals.push(p.normalize().to_array());
        colors.push(color);
    }

    let n = tile.corners.len() as u32;
    for i in 1..=n {
        let next = if i == n { 1 } else { i + 1 };
        indices.extend_from_slice(&[vertex_offset, vertex_offset + i, vertex_offset + next]);
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
