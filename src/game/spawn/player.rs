//! Spawn the player.

use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    game::{
        animation::PlayerAnimation,
        assets::{HandleMap, ImageKey},
        bullets::BulletSpawner,
        collision_groups::{ENEMY_GROUP, HIT_BOX_GROUP, PLAYER_GROUP},
        ghost::SpawnedGhost,
        layers,
        movement::{DashSettings, MovementController, MovementSettings, WrapWithinWindow},
    },
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_player);
    app.register_type::<Player>();
}

#[derive(Event, Debug)]
pub struct SpawnPlayer;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

pub const PLAYER_BASE_SPEED: f32 = 800.0;

fn spawn_player(
    _trigger: Trigger<SpawnPlayer>,
    mut commands: Commands,
    image_handles: Res<HandleMap<ImageKey>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // A texture atlas is a way to split one image with a grid into multiple sprites.
    // By attaching it to a [`SpriteBundle`] and providing an index, we can specify which section of the image we want to see.
    // We will use this to animate our player character. You can learn more about texture atlases in this example:
    // https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    let base_speed = PLAYER_BASE_SPEED;
    commands
        .spawn((
            Name::new("Player"),
            Player,
            SpriteBundle {
                sprite: Sprite {
                    anchor: bevy::sprite::Anchor::Custom(Vec2::new(0.0, -0.1)),
                    ..default()
                },
                texture: image_handles[&ImageKey::Ducky].clone_weak(),
                transform: Transform::from_scale(Vec2::splat(8.0).extend(1.0))
                    .with_translation(Vec2::ZERO.extend(layers::PLAYER)),
                ..Default::default()
            },
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: player_animation.get_atlas_index(),
            },
            MovementController::default(),
            MovementSettings {
                max_speed: base_speed,
            },
            DashSettings {
                intent_window: Duration::from_millis(100),
                distance: base_speed,
                time: Duration::from_millis(250),
            },
            WrapWithinWindow,
            player_animation,
            RigidBody::KinematicPositionBased,
            Collider::round_cuboid(6.0, 8.0, 50.0),
            StateScoped(Screen::Playing),
        ))
        .insert(CollisionGroups {
            memberships: PLAYER_GROUP,
            filters: Group::all(),
        })
        .observe(|trigger: Trigger<SpawnedGhost>, mut commands: Commands| {
            commands
                .entity(trigger.event().ghost)
                .insert(StateScoped(Screen::Playing));
        });
}
