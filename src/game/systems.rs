use bevy::prelude::*;
use bevy_fps_counter::FpsCounterText;

pub fn setup_controls_overlay(mut commands: Commands) {
    commands.spawn((
        Text::new("Drag left mouse: orbit\nMouse wheel: zoom\nWASD / arrows: orbit\nQ / E: zoom"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12.0),
            left: px(12.0),
            ..default()
        },
    ));
}

pub fn position_fps_counter(mut commands: Commands, query: Query<Entity, Added<FpsCounterText>>) {
    for entity in &query {
        commands.entity(entity).insert(Node {
            position_type: PositionType::Absolute,
            top: px(12.0),
            right: px(12.0),
            ..default()
        });
    }
}
