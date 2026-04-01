use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

use crate::camera::components::{MainCamera, OrbitCamera};
use crate::camera::resources::OrbitCameraSettings;

pub fn setup_scene(mut commands: Commands, camera_settings: Res<OrbitCameraSettings>) {
    let orbit_camera = OrbitCamera::new(camera_settings.focus, camera_settings.initial_position);

    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Transform::from_translation(orbit_camera.translation())
            .looking_at(orbit_camera.focus, Vec3::Y),
        orbit_camera,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 20_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, -0.8, 0.0)),
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
