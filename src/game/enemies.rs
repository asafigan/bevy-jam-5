use bevy::prelude::*;

use super::{health::Died, items::SpawnItem, spawn::player::Player};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Enemy>();
    app.observe(kill_enemy);
    app.add_systems(
        Update,
        (
            follow_player,
            push_enemies_away_from_each_other,
            push_enemies_away_from_player,
            push_enemies_away_from_each_other,
            push_enemies_away_from_player,
        )
            .chain(),
    );
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Enemy {
    pub max_speed: f32,
}

fn follow_player(
    time: Res<Time>,
    mut enemies: Query<(&Enemy, &mut Transform)>,
    players: Query<&GlobalTransform, With<Player>>,
) {
    if let Ok(player) = players.get_single() {
        let player_position = player.translation().truncate();
        for (enemy, mut transform) in &mut enemies {
            if let Ok((direction, length)) =
                Dir2::new_and_length(player_position - transform.translation.truncate())
            {
                let displacement = direction * (enemy.max_speed * time.delta_seconds()).min(length);
                transform.translation += displacement.extend(0.0);
            }
        }
    }
}

fn kill_enemy(
    trigger: Trigger<Died>,
    enemies: Query<&GlobalTransform, With<Enemy>>,
    mut commands: Commands,
) {
    if let Ok(transform) = enemies.get(trigger.entity()) {
        commands.trigger(SpawnItem {
            position: transform.translation().truncate(),
        });

        commands.entity(trigger.entity()).despawn_recursive();
    }
}

fn push_enemies_away_from_each_other(mut enemies: Query<&mut Transform, With<Enemy>>) {
    let mut combinations = enemies.iter_combinations_mut();
    while let Some([mut a, mut b]) = combinations.fetch_next() {
        let (direction, distance) =
            Dir2::new_and_length(a.translation.truncate() - b.translation.truncate())
                .unwrap_or((Dir2::NORTH, 0.0));

        let overlap = 100.0 - distance;
        if overlap > 0.0 {
            let delta = (direction * (overlap / 2.0)).extend(0.0);
            a.translation += delta;
            b.translation -= delta;
        }
    }
}

fn push_enemies_away_from_player(
    mut enemies: Query<&mut Transform, With<Enemy>>,
    players: Query<&GlobalTransform, With<Player>>,
) {
    let Ok(player) = players.get_single() else {
        return;
    };

    for mut enemy in &mut enemies {
        let (direction, distance) =
            Dir2::new_and_length(player.translation().truncate() - enemy.translation.truncate())
                .unwrap_or((Dir2::NORTH, 0.0));

        let overlap = 150.0 - distance;
        if overlap > 0.0 {
            let delta = (direction * overlap).extend(0.0);
            enemy.translation -= delta;
        }
    }
}
