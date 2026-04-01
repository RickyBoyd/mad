use bevy::prelude::*;

use crate::units::resources::{
    DEFAULT_SILO_SCALE, DEFAULT_SILO_SURFACE_OFFSET, DEFAULT_TANK_SCALE,
    DEFAULT_TANK_SURFACE_OFFSET,
};

#[derive(Message, Debug, Clone, Copy)]
pub struct SpawnPlanetTank {
    pub surface_direction: Vec3,
    pub heading_radians: f32,
    pub scale: f32,
    pub surface_offset: f32,
}

impl Default for SpawnPlanetTank {
    fn default() -> Self {
        Self {
            surface_direction: Vec3::Y,
            heading_radians: 0.0,
            scale: DEFAULT_TANK_SCALE,
            surface_offset: DEFAULT_TANK_SURFACE_OFFSET,
        }
    }
}

#[derive(Message, Debug, Clone, Copy)]
pub struct SpawnPlanetSilo {
    pub surface_direction: Vec3,
    pub heading_radians: f32,
    pub scale: f32,
    pub surface_offset: f32,
}

impl Default for SpawnPlanetSilo {
    fn default() -> Self {
        Self {
            surface_direction: Vec3::Y,
            heading_radians: 0.0,
            scale: DEFAULT_SILO_SCALE,
            surface_offset: DEFAULT_SILO_SURFACE_OFFSET,
        }
    }
}
