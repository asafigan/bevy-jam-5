use std::time::Duration;

use bevy::prelude::*;

use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<GhostSpawner>();
    app.add_systems(
        Update,
        (spawn_ghosts, tick_ghost_timer, despawn_ghosts, fade_ghosts)
            .chain()
            .in_set(AppSet::Update)
            .in_set(GhostSet::Update),
    );
}

#[derive(SystemSet, Clone, Debug, Hash, PartialEq, Eq)]
pub enum GhostSet {
    Update,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct GhostSpawner {
    pub timer: Timer,
    pub ghost_duration: Duration,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Ghost {
    starting_color: Color,
    timer: Timer,
}

#[derive(Event)]
pub struct SpawnedGhost {
    pub ghost: Entity,
}

fn spawn_ghosts(
    time: Res<Time>,
    mut spawners: Query<(
        Entity,
        &mut GhostSpawner,
        &GlobalTransform,
        &Sprite,
        &Handle<Image>,
        Option<&TextureAtlas>,
    )>,
    mut commands: Commands,
) {
    for (entity, mut spawner, global_transform, sprite, image, texture_atlas) in &mut spawners {
        spawner.timer.tick(time.delta());
        if spawner.timer.just_finished() {
            let mut transform = Transform::from(*global_transform);
            transform.translation.z -= 0.01;
            let mut ghost = commands.spawn((
                Ghost {
                    starting_color: sprite.color,
                    timer: Timer::new(spawner.ghost_duration, TimerMode::Once),
                },
                SpriteBundle {
                    sprite: sprite.clone(),
                    texture: image.clone(),
                    transform,
                    ..Default::default()
                },
            ));

            if let Some(texture_atlas) = texture_atlas {
                ghost.insert(texture_atlas.clone());
            }
            let ghost = ghost.id();
            commands.trigger_targets(SpawnedGhost { ghost }, entity);
        }
    }
}

fn tick_ghost_timer(time: Res<Time>, mut ghosts: Query<&mut Ghost>) {
    for mut ghost in &mut ghosts {
        ghost.timer.tick(time.delta());
    }
}

fn fade_ghosts(mut ghosts: Query<(&Ghost, &mut Sprite)>) {
    for (ghost, mut sprite) in &mut ghosts {
        sprite.color.set_alpha(ghost.timer.fraction_remaining());
    }
}

fn despawn_ghosts(ghosts: Query<(Entity, &Ghost)>, mut commands: Commands) {
    for (entity, ghost) in &ghosts {
        if ghost.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
