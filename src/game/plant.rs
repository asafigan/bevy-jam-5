use std::time::Duration;

use bevy::{
    color::palettes::css::{GREEN, GREEN_YELLOW},
    prelude::*,
};
use bevy_rapier2d::pipeline::CollisionEvent;

use super::spawn::{enemy::SpawnEnemy, player::Player};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (trigger_seed_events, growth).chain());
    app.observe(plant_seed).observe(finish_growing);
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Soil {
    pub plant: Option<Entity>,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Plant {
    growth_timer: Timer,
}

#[derive(Event)]
struct PlantSeed;

fn trigger_seed_events(
    mut events: EventReader<CollisionEvent>,
    players: Query<&Player>,
    soil: Query<&Soil>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let CollisionEvent::Started(a, b, _) = *event {
            let (_player, entity) = if players.contains(a) {
                (a, b)
            } else if players.contains(b) {
                (b, a)
            } else {
                continue;
            };

            if !soil.contains(entity) {
                continue;
            };

            commands.trigger_targets(PlantSeed, entity);
        }
    }
}

fn plant_seed(trigger: Trigger<PlantSeed>, mut soil: Query<&mut Soil>, mut commands: Commands) {
    let Ok(mut soil) = soil.get_mut(trigger.entity()) else {
        return;
    };

    if soil.plant.is_some() {
        return;
    }

    let plant = commands
        .spawn((
            SpriteBundle {
                transform: Transform::from_scale(Vec3::new(0.5, 1.0, 1.0))
                    .with_translation(Vec3::new(0.0, 0.5, 0.01)),
                ..default()
            },
            Plant {
                growth_timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
            },
        ))
        .set_parent(trigger.entity())
        .id();

    soil.plant = Some(plant);
}

#[derive(Event)]
struct FinishedGrowing;

fn growth(
    time: Res<Time>,
    mut plants: Query<(Entity, &mut Plant, &mut Sprite)>,
    mut commands: Commands,
) {
    for (entity, mut plant, mut sprite) in &mut plants {
        plant.growth_timer.tick(time.delta());
        sprite.color = GREEN_YELLOW
            .mix(&GREEN, plant.growth_timer.fraction())
            .into();
        if plant.growth_timer.finished() {
            commands.trigger_targets(FinishedGrowing, entity);
        }
    }
}

fn finish_growing(
    trigger: Trigger<FinishedGrowing>,
    parents: Query<&Parent>,
    mut soil: Query<(&mut Soil, &GlobalTransform)>,
    mut commands: Commands,
) {
    if let Ok(parent) = parents.get(trigger.entity()) {
        if let Ok((mut soil, global_transform)) = soil.get_mut(parent.get()) {
            soil.plant = None;
            commands.trigger(SpawnEnemy {
                position: global_transform.translation().truncate(),
            });
        }
    }
    commands.entity(trigger.entity()).despawn_recursive();
}
