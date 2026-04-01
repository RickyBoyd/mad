use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Copy)]
pub struct PlanetSettings {
    pub radius: f32,
    pub subdivisions: usize,
    pub plate_count: usize,
    pub tile_lift: f32,
}

impl Default for PlanetSettings {
    fn default() -> Self {
        Self {
            radius: 6.0,
            subdivisions: 5,
            plate_count: 13,
            tile_lift: 0.03,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct PlateGrowthSettings {
    pub batch_min: usize,
    pub batch_variation: usize,
}

impl Default for PlateGrowthSettings {
    fn default() -> Self {
        Self {
            batch_min: 3,
            batch_variation: 4,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct TerrainSettings {
    pub noise_frequency: f32,
    pub noise_octaves: usize,
    pub noise_lacunarity: f32,
    pub noise_persistence: f32,
    pub height_scale: f32,
    pub water_height: f32,
    pub surface_patch_resolution: usize,
    pub normal_sample_angle: f32,
    pub surface_detail_frequency: f32,
    pub surface_detail_radial_amplitude: f32,
    pub surface_detail_tangent_amplitude: f32,
}

impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            noise_frequency: 1.8,
            noise_octaves: 5,
            noise_lacunarity: 2.0,
            noise_persistence: 0.5,
            height_scale: 0.8,
            water_height: 0.0,
            surface_patch_resolution: 6,
            normal_sample_angle: 0.01,
            surface_detail_frequency: 18.0,
            surface_detail_radial_amplitude: 0.035,
            surface_detail_tangent_amplitude: 0.02,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct WorldSeed(pub u64);

impl Default for WorldSeed {
    fn default() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0);

        Self(seed)
    }
}
