use bevy::{color::palettes::css::YELLOW, prelude::*};

use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Item>();
    app.observe(spawn_item);
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
                .with_translation(trigger.event().position.extend(0.0)),
            sprite: Sprite {
                color: YELLOW.into(),
                ..default()
            },
            ..default()
        },
        StateScoped(Screen::Playing),
    ));
}
