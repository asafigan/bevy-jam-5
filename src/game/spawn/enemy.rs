use bevy::{color::palettes::css::WHITE, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
    game::{
        collision_groups::{ENEMY_GROUP, HIT_BOX_GROUP},
        enemies::Enemy,
        health::Health,
        layers,
        movement::WrapWithinWindow,
    },
    screen::Screen,
};

use super::player::PLAYER_BASE_SPEED;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_enemey);
}

#[derive(Event)]
pub struct SpawnEnemy {
    pub position: Vec2,
}

fn spawn_enemey(trigger: Trigger<SpawnEnemy>, mut commands: Commands) {
    commands.spawn((
        Name::new("Enemy"),
        Enemy {
            max_speed: PLAYER_BASE_SPEED,
        },
        Health::full(2.0),
        SpriteBundle {
            sprite: Sprite {
                color: WHITE.into(),
                ..default()
            },
            transform: Transform::from_scale(Vec2::splat(150.0).extend(1.0))
                .with_translation(trigger.event().position.extend(layers::ENEMIES)),
            ..default()
        },
        WrapWithinWindow,
        Collider::cuboid(0.5, 0.5),
        RigidBody::KinematicPositionBased,
        CollisionGroups {
            memberships: ENEMY_GROUP,
            filters: HIT_BOX_GROUP,
        },
        StateScoped(Screen::Playing),
    ));
}
