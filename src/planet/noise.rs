use bevy::prelude::*;

use crate::planet::random::splitmix64;
use crate::planet::resources::TerrainSettings;

#[derive(Debug, Clone, Copy)]
pub struct TerrainSample {
    pub terrain_height: f32,
    pub surface_height: f32,
    pub is_water: bool,
}

pub fn sample_terrain(direction: Vec3, settings: TerrainSettings, seed: u64) -> TerrainSample {
    let direction = direction.normalize_or_zero();
    let noise_value = sample_fractal_noise(
        direction,
        settings.noise_frequency,
        settings.noise_octaves,
        settings.noise_lacunarity,
        settings.noise_persistence,
        seed ^ 0xA5A5_5A5A_D3C1_B2E7,
    );
    let terrain_height = noise_value * settings.height_scale;
    let is_water = terrain_height <= settings.water_height;
    let surface_height = terrain_height.max(settings.water_height);

    TerrainSample {
        terrain_height,
        surface_height,
        is_water,
    }
}

pub fn sample_fractal_noise(
    direction: Vec3,
    frequency: f32,
    octaves: usize,
    lacunarity: f32,
    persistence: f32,
    seed: u64,
) -> f32 {
    let mut point = direction.normalize_or_zero() * frequency;
    let mut amplitude = 1.0;
    let mut total = 0.0;
    let mut amplitude_sum = 0.0;

    for octave in 0..octaves {
        let octave_seed = seed ^ (octave as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        total += value_noise_3d(point, octave_seed) * amplitude;
        amplitude_sum += amplitude;
        point *= lacunarity;
        amplitude *= persistence;
    }

    if amplitude_sum == 0.0 {
        0.0
    } else {
        total / amplitude_sum
    }
}

fn value_noise_3d(point: Vec3, seed: u64) -> f32 {
    let cell = point.floor();
    let local = point - cell;

    let x0 = cell.x as i32;
    let y0 = cell.y as i32;
    let z0 = cell.z as i32;

    let tx = smoothstep(local.x);
    let ty = smoothstep(local.y);
    let tz = smoothstep(local.z);

    let c000 = lattice_value(seed, x0, y0, z0);
    let c100 = lattice_value(seed, x0 + 1, y0, z0);
    let c010 = lattice_value(seed, x0, y0 + 1, z0);
    let c110 = lattice_value(seed, x0 + 1, y0 + 1, z0);
    let c001 = lattice_value(seed, x0, y0, z0 + 1);
    let c101 = lattice_value(seed, x0 + 1, y0, z0 + 1);
    let c011 = lattice_value(seed, x0, y0 + 1, z0 + 1);
    let c111 = lattice_value(seed, x0 + 1, y0 + 1, z0 + 1);

    let x00 = lerp(c000, c100, tx);
    let x10 = lerp(c010, c110, tx);
    let x01 = lerp(c001, c101, tx);
    let x11 = lerp(c011, c111, tx);

    let y0 = lerp(x00, x10, ty);
    let y1 = lerp(x01, x11, ty);

    lerp(y0, y1, tz)
}

fn lattice_value(seed: u64, x: i32, y: i32, z: i32) -> f32 {
    let hashed = splitmix64(
        seed ^ (x as i64 as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ (y as i64 as u64).wrapping_mul(0x94D0_49BB_1331_11EB)
            ^ (z as i64 as u64).wrapping_mul(0xD6E8_FD9D_3F5A_BC31),
    );

    ((hashed as f64 / u64::MAX as f64) * 2.0 - 1.0) as f32
}

fn smoothstep(value: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
