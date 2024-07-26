use std::time::Duration;

use bevy::{
    color::palettes::css::{BLUE, GREEN},
    prelude::*,
    render::mesh::CircleMeshBuilder,
    sprite::Mesh2dHandle,
};

use bevy_rapier2d::prelude::*;

use super::{
    bullets::BulletSpawner,
    collision_groups::{ENEMY_GROUP, HIT_BOX_GROUP, SOIL_GROUP},
    plant::{Planter, Water},
    spawn::player::Player,
};

pub(super) fn plugin(app: &mut App) {
    app.observe(switch_tool);
    app.add_systems(Startup, init_tool_assets);
    app.add_systems(Update, controls);
}

#[derive(Event)]
pub struct SwitchTool {
    pub tool_kind: ToolKind,
}

pub enum ToolKind {
    SeedPlanter,
    Gun,
    Water,
}

#[derive(Resource)]
struct ToolAssets {
    circle_mesh: Mesh2dHandle,
    planter_material: Handle<ColorMaterial>,
    water_material: Handle<ColorMaterial>,
}

fn init_tool_assets(
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    commands.insert_resource(ToolAssets {
        circle_mesh: Mesh2dHandle(meshes.add(CircleMeshBuilder::new(0.5, 100).build())),
        planter_material: color_materials.add(ColorMaterial::from_color(GREEN.with_alpha(0.3))),
        water_material: color_materials.add(ColorMaterial::from_color(BLUE.with_alpha(0.3))),
    });
}

fn controls(
    input: Res<ButtonInput<KeyCode>>,
    players: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    let Ok(player) = players.get_single() else {
        return;
    };

    let tool_kind = if input.just_pressed(KeyCode::Digit1) {
        ToolKind::SeedPlanter
    } else if input.just_pressed(KeyCode::Digit2) {
        ToolKind::Water
    } else if input.just_pressed(KeyCode::Digit3) {
        ToolKind::Gun
    } else {
        return;
    };

    commands.trigger_targets(SwitchTool { tool_kind }, player);
}

fn switch_tool(
    trigger: Trigger<SwitchTool>,
    planter_assets: Res<ToolAssets>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity())
        .despawn_descendants()
        .with_children(|children| {
            match trigger.event().tool_kind {
                ToolKind::Gun => children.spawn((
                    SpatialBundle::default(),
                    BulletSpawner {
                        bullet_damage: 1.0,
                        bullet_speed: 2000.0,
                        timer: Timer::new(Duration::from_millis(100), TimerMode::Repeating),
                        bullet_time_to_live: Duration::from_secs(5),
                        bullet_radius: 25.0,
                        collision_groups: CollisionGroups {
                            memberships: HIT_BOX_GROUP,
                            filters: ENEMY_GROUP,
                        },
                    },
                )),
                ToolKind::SeedPlanter => children.spawn((
                    ColorMesh2dBundle {
                        mesh: planter_assets.circle_mesh.clone(),
                        material: planter_assets.planter_material.clone(),
                        transform: Transform::from_scale(Vec2::splat(50.0).extend(1.0))
                            .with_translation(Vec2::ZERO.extend(-0.01)),
                        ..default()
                    },
                    Planter,
                    Sensor,
                    Collider::ball(0.5),
                    CollisionGroups {
                        memberships: Group::all(),
                        filters: SOIL_GROUP,
                    },
                )),
                ToolKind::Water => children.spawn((
                    ColorMesh2dBundle {
                        mesh: planter_assets.circle_mesh.clone(),
                        material: planter_assets.water_material.clone(),
                        transform: Transform::from_scale(Vec2::splat(50.0).extend(1.0))
                            .with_translation(Vec2::ZERO.extend(-0.01)),
                        ..default()
                    },
                    Water,
                    Sensor,
                    Collider::ball(0.5),
                    CollisionGroups {
                        memberships: Group::all(),
                        filters: SOIL_GROUP,
                    },
                )),
            };
        });
}
