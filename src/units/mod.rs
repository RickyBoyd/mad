pub mod components;
pub mod messages;
pub mod resources;
pub mod systems;

use bevy::prelude::*;

use self::messages::{SpawnPlanetSilo, SpawnPlanetTank};
use self::resources::UnitSpawnCounter;

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnitSpawnCounter>()
            .add_message::<SpawnPlanetTank>()
            .add_message::<SpawnPlanetSilo>()
            .add_systems(
                Startup,
                (systems::load_unit_assets, systems::spawn_demo_units),
            )
            .add_systems(
                Update,
                (systems::spawn_planet_tanks, systems::spawn_planet_silos),
            );
    }
}
