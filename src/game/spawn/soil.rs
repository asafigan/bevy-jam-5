use crate::{game::plant::Soil, screen::Screen};
use bevy::{color::palettes::css::BROWN, prelude::*};
use bevy_rapier2d::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_soil);
}

#[derive(Event, Debug)]
pub struct SpawnSoil {
    pub position: Vec2,
}

fn spawn_soil(trigger: Trigger<SpawnSoil>, mut commands: Commands) {
    commands.spawn((
        Name::new("Soil"),
        Soil::default(),
        SpriteBundle {
            sprite: Sprite {
                color: BROWN.into(),
                ..default()
            },
            transform: Transform::from_scale(Vec2::splat(200.0).extend(1.0))
                .with_translation(trigger.event().position.extend(-0.2)),
            ..default()
        },
        Collider::cuboid(0.5, 0.5),
        Sensor,
        StateScoped(Screen::Playing),
    ));
}