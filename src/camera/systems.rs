use std::f32::consts::TAU;

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

use crate::camera::components::{MainCamera, OrbitCamera, OrbitingSun};
use crate::camera::resources::{OrbitCameraSettings, SunOrbitSettings};

pub fn setup_scene(
    mut commands: Commands,
    camera_settings: Res<OrbitCameraSettings>,
    sun_settings: Res<SunOrbitSettings>,
) {
    let orbit_camera = OrbitCamera::new(camera_settings.focus, camera_settings.initial_position);

    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Transform::from_translation(orbit_camera.translation())
            .looking_at(orbit_camera.focus, Vec3::Y),
        orbit_camera,
    ));

    let sun_position = sun_position(*sun_settings, sun_settings.initial_angle);
    commands.spawn((
        OrbitingSun {
            angle: sun_settings.initial_angle,
        },
        DirectionalLight {
            illuminance: 20_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(sun_position).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn orbit_camera_controls(
    mut camera: Single<(&mut OrbitCamera, &mut Transform), With<MainCamera>>,
    camera_settings: Res<OrbitCameraSettings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
) {
    let (camera, transform) = &mut *camera;

    if mouse_buttons.pressed(MouseButton::Left) {
        camera.yaw -= mouse_motion.delta.x * camera_settings.rotation_sensitivity;
        camera.pitch -= mouse_motion.delta.y * camera_settings.rotation_sensitivity;
    }

    let orbit_speed = camera_settings.keyboard_orbit_speed * time.delta_secs();
    let zoom_speed = camera_settings.keyboard_zoom_speed * time.delta_secs();

    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        camera.yaw += orbit_speed;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        camera.yaw -= orbit_speed;
    }
    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        camera.pitch += orbit_speed * camera_settings.keyboard_pitch_scale;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        camera.pitch -= orbit_speed * camera_settings.keyboard_pitch_scale;
    }
    if keyboard.pressed(KeyCode::KeyQ) {
        camera.radius -= zoom_speed;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        camera.radius += zoom_speed;
    }

    let scroll_zoom = 1.0 - mouse_scroll.delta.y * camera_settings.scroll_zoom_factor;
    if scroll_zoom > 0.0 {
        camera.radius *= scroll_zoom;
    }

    camera.pitch = camera
        .pitch
        .clamp(camera_settings.min_pitch, camera_settings.max_pitch);
    camera.radius = camera
        .radius
        .clamp(camera_settings.min_radius, camera_settings.max_radius);

    transform.translation = camera.translation();
    transform.look_at(camera.focus, Vec3::Y);
}

pub fn orbit_directional_light(
    mut sun: Single<(&mut OrbitingSun, &mut Transform), With<DirectionalLight>>,
    sun_settings: Res<SunOrbitSettings>,
    time: Res<Time>,
) {
    let (sun, transform) = &mut *sun;

    sun.angle = (sun.angle + TAU * time.delta_secs() / sun_settings.period_seconds).rem_euclid(TAU);
    let position = sun_position(*sun_settings, sun.angle);
    transform.translation = position;
    transform.look_at(Vec3::ZERO, Vec3::Y);
}

fn sun_position(settings: SunOrbitSettings, angle: f32) -> Vec3 {
    let orbit = Vec3::new(
        settings.orbit_radius * angle.cos(),
        0.0,
        settings.orbit_radius * angle.sin(),
    );

    Quat::from_rotation_x(settings.tilt) * orbit
}
