pub mod components;
pub mod resources;
pub mod systems;

use bevy::prelude::*;

use crate::game::states::GameState;

use self::resources::OrbitCameraSettings;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OrbitCameraSettings>()
            .add_systems(Startup, systems::setup_scene)
            .add_systems(
                Update,
                systems::orbit_camera_controls.run_if(in_state(GameState::InGame)),
            );
    }
}
