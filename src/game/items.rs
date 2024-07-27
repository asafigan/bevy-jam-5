use bevy::a11y::accesskit::TextSelection;
use bevy::color::palettes::css::WHITE;
use bevy::{color::palettes::css::YELLOW, prelude::*};

use crate::screen::Screen;
use crate::ui::prelude::*;

use super::spawn::player::Player;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<(Item, DrawToward, Wallet)>();
    app.init_resource::<Wallet>();
    app.observe(spawn_item);
    app.add_systems(
        Update,
        (
            draw_items_to_player,
            draw_toward,
            player_collects_items,
            update_wallet_display,
        )
            .chain(),
    );
    app.add_systems(
        OnEnter(Screen::Playing),
        (reset_wallet, setup_wallet_display),
    );
}

#[derive(Event)]
pub struct SpawnItem {
    pub position: Vec2,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Item;

fn spawn_item(trigger: Trigger<SpawnItem>, mut commands: Commands) {
    commands.spawn((
        Name::new("Item"),
        Item,
        SpriteBundle {
            transform: Transform::from_scale(Vec2::splat(25.0).extend(1.0))
                .with_translation(trigger.event().position.extend(-0.1)),
            sprite: Sprite {
                color: YELLOW.into(),
                ..default()
            },
            ..default()
        },
        StateScoped(Screen::Playing),
    ));
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DrawToward {
    target: Entity,
    speed: f32,
    acceleration: f32,
}

fn draw_items_to_player(
    players: Query<(Entity, &GlobalTransform), With<Player>>,
    mut items: Query<(Entity, &GlobalTransform), (With<Item>, Without<DrawToward>)>,
    mut commands: Commands,
) {
    let Ok((player, player_transform)) = players.get_single() else {
        return;
    };

    for (item, transform) in &mut items {
        if (player_transform.translation() - transform.translation())
            .truncate()
            .length()
            <= 1000.0
        {
            commands.entity(item).insert(DrawToward {
                target: player,
                speed: 0.0,
                acceleration: 10000.0,
            });
        }
    }
}

fn draw_toward(
    time: Res<Time>,
    mut entities: Query<(&mut DrawToward, &mut Transform)>,
    global_transforms: Query<&GlobalTransform>,
) {
    for (mut draw_toward, mut transform) in &mut entities {
        let last_speed = draw_toward.speed;
        draw_toward.speed += draw_toward.acceleration * time.delta_seconds();
        let average_speed = (last_speed + draw_toward.speed) / 2.0;
        if let Ok(global_transform) = global_transforms.get(draw_toward.target) {
            if let Ok((direction, length)) = Dir2::new_and_length(
                (global_transform.translation() - transform.translation).truncate(),
            ) {
                let distance = (average_speed * time.delta_seconds()).min(length);

                transform.translation += (direction * distance).extend(0.0);
            }
        }
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct Wallet {
    pub amount: u32,
}

fn player_collects_items(
    mut wallet: ResMut<Wallet>,
    players: Query<&GlobalTransform, With<Player>>,
    items: Query<(Entity, &GlobalTransform), With<Item>>,
    mut commands: Commands,
) {
    let Ok(player) = players.get_single() else {
        return;
    };

    for (item, transform) in &items {
        if (player.translation() - transform.translation())
            .truncate()
            .length()
            <= 50.0
        {
            commands.entity(item).despawn_recursive();
            wallet.amount += 1;
        }
    }
}

fn reset_wallet(mut wallet: ResMut<Wallet>) {
    wallet.amount = 0;
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct WalletDisplay;

fn update_wallet_display(wallet: Res<Wallet>, mut displays: Query<&mut Text, With<WalletDisplay>>) {
    for mut text in &mut displays {
        text.sections[1].value = wallet.amount.to_string();
    }
}

fn setup_wallet_display(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Top Display"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Row,
                    row_gap: Val::Px(10.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
            StateScoped(Screen::Playing),
        ))
        .with_children(|children| {
            children.spawn((
                TextBundle {
                    text: Text::from_sections([
                        TextSection::new(
                            "$",
                            TextStyle {
                                font: default(),
                                font_size: 100.0,
                                color: WHITE.into(),
                            },
                        ),
                        TextSection::new(
                            "0",
                            TextStyle {
                                font: default(),
                                font_size: 100.0,
                                color: WHITE.into(),
                            },
                        ),
                    ]),
                    ..default()
                },
                WalletDisplay,
            ));
        });
}
