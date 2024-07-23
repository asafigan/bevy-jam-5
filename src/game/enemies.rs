use bevy::prelude::*;

use super::spawn::player::Player;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Enemy>();
    app.add_systems(Update, follow_player);
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