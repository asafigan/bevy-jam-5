use std::time::Duration;

use bevy::{
    color::palettes::css::{GREEN, GREEN_YELLOW, SADDLE_BROWN, SANDY_BROWN},
    prelude::*,
};
use bevy_rapier2d::pipeline::CollisionEvent;

use super::spawn::enemy::SpawnEnemy;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (trigger_seed_events, trigger_water_event, growth, soil_color).chain(),
    );
    app.observe(plant_seed)
        .observe(water_soil)
        .observe(finish_growing);
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Soil {
    pub plant: Option<Entity>,
    pub state: SoilState,
}

#[derive(Default, Reflect, PartialEq, Eq)]
pub enum SoilState {
    #[default]
    Dry,
    Wet,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Plant {
    growth_timer: Timer,
    stages: u8,
    current_stage: u8,
}

#[derive(Event)]
struct PlantSeed;

#[derive(Component)]
pub struct Planter;

fn trigger_seed_events(
    mut events: EventReader<CollisionEvent>,
    planters: Query<&Planter>,
    soil: Query<&Soil>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let CollisionEvent::Started(a, b, _) = *event {
            let (_sensor, entity) = if planters.contains(a) {
                (a, b)
            } else if planters.contains(b) {
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

    let stages = 2;

    let plant = commands
        .spawn((
            SpriteBundle {
                transform: Transform::from_scale(Vec3::new(0.5, 1.0 / stages as f32, 1.0))
                    .with_translation(Vec3::new(0.0, 0.0, 0.01)),
                sprite: Sprite {
                    color: GREEN_YELLOW.into(),
                    anchor: bevy::sprite::Anchor::BottomCenter,
                    ..default()
                },
                ..default()
            },
            Plant {
                growth_timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
                stages,
                current_stage: 0,
            },
        ))
        .set_parent(trigger.entity())
        .id();

    soil.plant = Some(plant);
}

#[derive(Component)]
pub struct Water;

#[derive(Event)]
struct WaterSoil;

fn trigger_water_event(
    mut events: EventReader<CollisionEvent>,
    water: Query<&Water>,
    soil: Query<&Soil>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let CollisionEvent::Started(a, b, _) = *event {
            let (_sensor, entity) = if water.contains(a) {
                (a, b)
            } else if water.contains(b) {
                (b, a)
            } else {
                continue;
            };

            if !soil.contains(entity) {
                continue;
            };

            commands.trigger_targets(WaterSoil, entity);
        }
    }
}

fn water_soil(trigger: Trigger<WaterSoil>, mut soil: Query<&mut Soil>) {
    let Ok(mut soil) = soil.get_mut(trigger.entity()) else {
        return;
    };

    soil.state = SoilState::Wet;
}

#[derive(Event)]
struct FinishedGrowing;

fn growth(
    time: Res<Time>,
    mut plants: Query<(Entity, &mut Plant, &mut Sprite, &Parent, &mut Transform)>,
    mut soil: Query<&mut Soil>,
    mut commands: Commands,
) {
    for (entity, mut plant, mut sprite, parent, mut transform) in &mut plants {
        let Ok(mut soil) = soil.get_mut(parent.get()) else {
            continue;
        };

        if soil.state != SoilState::Wet {
            continue;
        }

        plant.growth_timer.tick(time.delta());
        sprite.color = GREEN_YELLOW
            .mix(
                &GREEN,
                (plant.current_stage as f32 + plant.growth_timer.fraction()) / plant.stages as f32,
            )
            .into();
        if plant.growth_timer.finished() {
            soil.state = SoilState::Dry;
            plant.current_stage += 1;
            transform.scale.y = (plant.current_stage + 1) as f32 / plant.stages as f32;
            plant.growth_timer.reset();
            if plant.current_stage >= plant.stages {
                commands.trigger_targets(FinishedGrowing, entity);
            }
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

fn soil_color(mut soil: Query<(&Soil, &mut Sprite)>) {
    for (soil, mut sprite) in &mut soil {
        sprite.color = match soil.state {
            SoilState::Dry => SANDY_BROWN,
            SoilState::Wet => SADDLE_BROWN,
        }
        .into();
    }
}
