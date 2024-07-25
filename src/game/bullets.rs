use std::time::Duration;

use bevy::{
    color::palettes::css::WHITE, prelude::*, render::mesh::CircleMeshBuilder, sprite::Mesh2dHandle,
};
use bevy_rapier2d::prelude::*;

use crate::screen::Screen;

use super::health::Damage;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_bullet);
    app.add_systems(Startup, init_bullet_assets);
    app.add_systems(
        Update,
        (fire_bullets, hit_test_bullets, move_bullets).chain(),
    );
    app.add_systems(PreUpdate, time_to_live);
}

#[derive(Resource)]
struct BulletAssets {
    mesh: Mesh2dHandle,
    material: Handle<ColorMaterial>,
}

fn init_bullet_assets(
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    commands.insert_resource(BulletAssets {
        mesh: Mesh2dHandle(meshes.add(CircleMeshBuilder::new(1.0, 100).build())),
        material: color_materials.add(ColorMaterial::from_color(WHITE)),
    });
}

#[derive(Component)]
struct Bullet {
    damage: f32,
    velocity: Vec2,
    collision_groups: CollisionGroups,
    collider: Collider,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct TimeToLive {
    timer: Timer,
}

#[derive(Component)]
pub struct BulletSpawner {
    pub bullet_damage: f32,
    pub bullet_speed: f32,
    pub bullet_radius: f32,
    pub bullet_time_to_live: Duration,
    pub timer: Timer,
    pub collision_groups: CollisionGroups,
}

#[derive(Event)]
pub struct SpawnBullet {
    pub damage: f32,
    pub position: Vec2,
    pub direction: Dir2,
    pub speed: f32,
    pub time_to_live: Duration,
    pub collision_groups: CollisionGroups,
    pub radius: f32,
}

fn spawn_bullet(
    trigger: Trigger<SpawnBullet>,
    bullet_assets: Res<BulletAssets>,
    mut commands: Commands,
) {
    let diameter = trigger.event().radius * 2.0;
    commands.spawn((
        Bullet {
            damage: trigger.event().damage,
            velocity: trigger.event().direction * trigger.event().speed,
            collision_groups: trigger.event().collision_groups,
            collider: Collider::ball(trigger.event().radius),
        },
        TimeToLive {
            timer: Timer::new(trigger.event().time_to_live, TimerMode::Once),
        },
        ColorMesh2dBundle {
            mesh: bullet_assets.mesh.clone(),
            material: bullet_assets.material.clone(),
            transform: Transform::from_scale(Vec3::new(diameter, diameter, 1.0))
                .with_translation(trigger.event().position.extend(0.1)),
            ..default()
        },
        StateScoped(Screen::Playing),
    ));
}

fn fire_bullets(
    time: Res<Time>,
    mut spawners: Query<(&GlobalTransform, &mut BulletSpawner)>,
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
) {
    for (global_transform, mut spawner) in &mut spawners {
        spawner.timer.tick(time.delta());
        if spawner.timer.just_finished() {
            let filter = bevy_rapier2d::pipeline::QueryFilter::from(spawner.collision_groups);
            let position = global_transform.translation().truncate();
            if let Some((_entity, projection)) =
                rapier_context.project_point(position, false, filter)
            {
                let direction = Dir2::new(projection.point - position).unwrap_or(Dir2::NORTH);
                commands.trigger(SpawnBullet {
                    position,
                    damage: spawner.bullet_damage,
                    direction,
                    speed: spawner.bullet_speed,
                    time_to_live: spawner.bullet_time_to_live,
                    collision_groups: spawner.collision_groups,
                    radius: spawner.bullet_radius.clone(),
                });
            }
        }
    }
}

fn hit_test_bullets(
    time: Res<Time>,
    bullets: Query<(Entity, &Transform, &Bullet)>,
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
) {
    for (entity, global_transform, bullet) in &bullets {
        let position = global_transform.translation.truncate();
        let rotation = 0.0; // rotation in radians
        let options = ShapeCastOptions {
            max_time_of_impact: bullet.velocity.length() * time.delta_seconds(),
            target_distance: 0.0,
            stop_at_penetration: true,
            compute_impact_geometry_on_penetration: false,
        };
        if let Some((hit_entity, _toi)) = rapier_context.cast_shape(
            position,
            rotation,
            bullet.velocity.normalize(),
            &bullet.collider,
            options,
            bullet.collision_groups.into(),
        ) {
            commands.entity(entity).despawn_recursive();
            commands.trigger_targets(
                Damage {
                    amount: bullet.damage,
                },
                hit_entity,
            );
        }
    }
}

fn move_bullets(time: Res<Time>, mut bullets: Query<(&mut Transform, &Bullet)>) {
    for (mut transform, bullet) in &mut bullets {
        transform.translation += (bullet.velocity * time.delta_seconds()).extend(0.0);
    }
}

fn time_to_live(
    time: Res<Time>,
    mut query: Query<(Entity, &mut TimeToLive)>,
    mut commands: Commands,
) {
    for (entity, mut time_to_live) in &mut query {
        time_to_live.timer.tick(time.delta());
        if time_to_live.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
