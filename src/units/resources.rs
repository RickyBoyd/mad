use bevy::prelude::*;

pub const CHALLENGER_2_MODEL_PATH: &str = "challenger2_lowpoly.glb";
pub const MISSILE_SILO_MODEL_PATH: &str = "missile_silo.glb";
pub const DEFAULT_TANK_SCALE: f32 = 0.08;
pub const DEFAULT_TANK_SURFACE_OFFSET: f32 = 0.03;
pub const DEFAULT_SILO_SCALE: f32 = 0.12;
pub const DEFAULT_SILO_SURFACE_OFFSET: f32 = 0.02;

#[derive(Resource, Debug, Clone)]
pub struct UnitAssets {
    pub challenger2_scene: Handle<Scene>,
    pub missile_silo_scene: Handle<Scene>,
}

#[derive(Resource, Debug, Default)]
pub struct UnitSpawnCounter(pub u64);
