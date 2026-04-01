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
            subdivisions: 8,
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
