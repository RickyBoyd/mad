use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Copy)]
pub struct OrbitCameraSettings {
    pub focus: Vec3,
    pub initial_position: Vec3,
    pub rotation_sensitivity: f32,
    pub keyboard_orbit_speed: f32,
    pub keyboard_pitch_scale: f32,
    pub keyboard_zoom_speed: f32,
    pub scroll_zoom_factor: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub min_radius: f32,
    pub max_radius: f32,
}

impl Default for OrbitCameraSettings {
    fn default() -> Self {
        let pitch_limit = FRAC_PI_2 - 0.05;

        Self {
            focus: Vec3::ZERO,
            initial_position: Vec3::new(0.0, 8.0, 18.0),
            rotation_sensitivity: 0.005,
            keyboard_orbit_speed: 1.8,
            keyboard_pitch_scale: 0.6,
            keyboard_zoom_speed: 9.0,
            scroll_zoom_factor: 0.08,
            min_pitch: -pitch_limit,
            max_pitch: pitch_limit,
            min_radius: 7.0,
            max_radius: 35.0,
        }
    }
}
