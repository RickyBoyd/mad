pub mod assets;
pub mod components;
pub mod generation;
pub mod mesh;
pub mod noise;
pub mod random;
pub mod resources;
pub mod systems;

use bevy::prelude::*;

use self::resources::{PlanetSettings, PlateGrowthSettings, TerrainSettings, WorldSeed};

pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlanetSettings>()
            .init_resource::<PlateGrowthSettings>()
            .init_resource::<TerrainSettings>()
            .init_resource::<WorldSeed>()
            .add_systems(Startup, systems::setup_planet);
    }
}
