pub mod states;
pub mod systems;

use bevy::prelude::*;
use bevy_fps_counter::FpsCounterPlugin;

use crate::camera::CameraPlugin;
use crate::planet::PlanetPlugin;

use self::states::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((CameraPlugin, PlanetPlugin, FpsCounterPlugin))
            .add_systems(Startup, systems::setup_controls_overlay)
            .add_systems(Update, systems::position_fps_counter);
    }
}
