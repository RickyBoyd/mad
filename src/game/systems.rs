use bevy::prelude::*;

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
