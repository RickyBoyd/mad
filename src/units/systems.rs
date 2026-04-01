use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::planet::mesh::sample_surface_frame;
use crate::planet::random::Random;
use crate::planet::resources::{PlanetSettings, TerrainSettings, WorldSeed};
use crate::units::components::{PlanetSilo, PlanetTank};
use crate::units::messages::{SpawnPlanetSilo, SpawnPlanetTank};
use crate::units::resources::{
    CHALLENGER_2_MODEL_PATH, MISSILE_SILO_MODEL_PATH, UnitAssets, UnitSpawnCounter,
};

const TANK_MODEL_ROTATION_OFFSET: f32 = -FRAC_PI_2;
const DEMO_TANK_COUNT: usize = 10;
const DEMO_SILO_COUNT: usize = 5;

pub fn load_unit_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(UnitAssets {
        challenger2_scene: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(CHALLENGER_2_MODEL_PATH)),
        missile_silo_scene: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(MISSILE_SILO_MODEL_PATH)),
    });
}

pub fn spawn_demo_units(
    mut tank_writer: MessageWriter<SpawnPlanetTank>,
    mut silo_writer: MessageWriter<SpawnPlanetSilo>,
    world_seed: Res<WorldSeed>,
) {
    let mut rng = Random::new(world_seed.0 ^ 0xB1A3_7F52_8D61_C4E9);

    for _ in 0..DEMO_TANK_COUNT {
        tank_writer.write(SpawnPlanetTank {
            surface_direction: random_unit_vector(&mut rng),
            heading_radians: rng.next_f32() * TAU,
            ..default()
        });
    }

    for _ in 0..DEMO_SILO_COUNT {
        silo_writer.write(SpawnPlanetSilo {
            surface_direction: random_unit_vector(&mut rng),
            heading_radians: rng.next_f32() * TAU,
            ..default()
        });
    }
}

pub fn spawn_planet_tanks(
    mut commands: Commands,
    mut tank_requests: MessageReader<SpawnPlanetTank>,
    unit_assets: Res<UnitAssets>,
    mut spawn_counter: ResMut<UnitSpawnCounter>,
    planet_settings: Res<PlanetSettings>,
    terrain_settings: Res<TerrainSettings>,
    world_seed: Res<WorldSeed>,
) {
    for request in tank_requests.read() {
        let up = request.surface_direction.normalize_or_zero();
        if up.length_squared() <= f32::EPSILON {
            continue;
        }

        let surface = sample_surface_frame(
            up,
            planet_settings.radius,
            *terrain_settings,
            world_seed.0,
            planet_settings.tile_lift,
        );
        let forward = tangent_direction_from_heading(surface.up, request.heading_radians);
        let rotation = Transform::from_translation(Vec3::ZERO)
            .looking_to(forward, surface.up)
            .rotation
            * Quat::from_rotation_x(TANK_MODEL_ROTATION_OFFSET);
        let unit_id = spawn_counter.0;
        spawn_counter.0 += 1;

        commands.spawn((
            PlanetTank,
            Name::new(format!("Challenger2 Tank {unit_id}")),
            SceneRoot(unit_assets.challenger2_scene.clone()),
            Transform {
                translation: surface.position + surface.up * request.surface_offset,
                rotation,
                scale: Vec3::splat(request.scale),
            },
        ));
    }
}

pub fn spawn_planet_silos(
    mut commands: Commands,
    mut silo_requests: MessageReader<SpawnPlanetSilo>,
    unit_assets: Res<UnitAssets>,
    mut spawn_counter: ResMut<UnitSpawnCounter>,
    planet_settings: Res<PlanetSettings>,
    terrain_settings: Res<TerrainSettings>,
    world_seed: Res<WorldSeed>,
) {
    for request in silo_requests.read() {
        let up = request.surface_direction.normalize_or_zero();
        if up.length_squared() <= f32::EPSILON {
            continue;
        }

        let surface = sample_surface_frame(
            up,
            planet_settings.radius,
            *terrain_settings,
            world_seed.0,
            planet_settings.tile_lift,
        );
        let forward = tangent_direction_from_heading(surface.up, request.heading_radians);
        let rotation = Transform::from_translation(Vec3::ZERO)
            .looking_to(forward, surface.up)
            .rotation
            * Quat::from_rotation_x(TANK_MODEL_ROTATION_OFFSET);
        let unit_id = spawn_counter.0;
        spawn_counter.0 += 1;

        commands.spawn((
            PlanetSilo,
            Name::new(format!("Missile Silo {unit_id}")),
            SceneRoot(unit_assets.missile_silo_scene.clone()),
            Transform {
                translation: surface.position + surface.up * request.surface_offset,
                rotation,
                scale: Vec3::splat(request.scale),
            },
        ));
    }
}

fn tangent_direction_from_heading(up: Vec3, heading_radians: f32) -> Vec3 {
    let reference = if up.y.abs() < 0.99 { Vec3::Y } else { Vec3::X };
    let tangent_x = up.cross(reference).normalize_or_zero();
    let tangent_y = tangent_x.cross(up).normalize_or_zero();

    (tangent_x * heading_radians.cos() + tangent_y * heading_radians.sin()).normalize_or_zero()
}

fn random_unit_vector(rng: &mut Random) -> Vec3 {
    let z = rng.next_f32() * 2.0 - 1.0;
    let azimuth = rng.next_f32() * TAU;
    let radius = (1.0 - z * z).sqrt();

    Vec3::new(radius * azimuth.cos(), z, radius * azimuth.sin()).normalize_or_zero()
}
