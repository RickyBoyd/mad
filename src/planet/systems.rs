use bevy::math::primitives::Sphere;
use bevy::prelude::*;

use crate::planet::assets::build_planet_assets;
use crate::planet::components::{PlanetBase, PlanetTiles};
use crate::planet::generation::{build_icosphere, build_plates, build_tile_neighbors, build_tiles};
use crate::planet::mesh::tile_field_to_mesh;
use crate::planet::resources::{PlanetSettings, PlateGrowthSettings, WorldSeed};

pub fn setup_planet(
    mut commands: Commands,
    planet_settings: Res<PlanetSettings>,
    growth_settings: Res<PlateGrowthSettings>,
    world_seed: Res<WorldSeed>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tri_mesh = build_icosphere(planet_settings.radius, planet_settings.subdivisions);
    let tiles = build_tiles(&tri_mesh, planet_settings.radius);
    let tile_neighbors = build_tile_neighbors(&tri_mesh);
    let plate_ids = build_plates(
        &tiles,
        &tile_neighbors,
        planet_settings.plate_count,
        *growth_settings,
        world_seed.0,
    );
    let planet_assets = build_planet_assets(&mut materials);

    commands.spawn((
        PlanetBase,
        Mesh3d(meshes.add(Mesh::from(Sphere {
            radius: planet_settings.radius,
        }))),
        MeshMaterial3d(planet_assets.base_sphere_material),
        Transform::default(),
    ));

    commands.spawn((
        PlanetTiles,
        Mesh3d(meshes.add(tile_field_to_mesh(
            &tiles,
            &plate_ids,
            world_seed.0,
            planet_settings.tile_lift,
        ))),
        MeshMaterial3d(planet_assets.tile_field_material),
        Transform::default(),
    ));
}
