use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct PlanetAssets {
    pub base_sphere_material: Handle<StandardMaterial>,
    pub tile_field_material: Handle<StandardMaterial>,
}

pub fn build_planet_assets(materials: &mut Assets<StandardMaterial>) -> PlanetAssets {
    PlanetAssets {
        base_sphere_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.08, 0.15),
            perceptual_roughness: 1.0,
            ..default()
        }),
        tile_field_material: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 1.0),
            perceptual_roughness: 0.95,
            ..default()
        }),
    }
}
