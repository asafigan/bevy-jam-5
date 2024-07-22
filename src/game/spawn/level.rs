//! Spawn the main level by triggering other observers.

use bevy::prelude::*;

use super::{player::SpawnPlayer, soil::SpawnSoil};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(_trigger: Trigger<SpawnLevel>, mut commands: Commands) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    commands.trigger(SpawnPlayer);

    for x in -5..=5 {
        for y in -5..=5 {
            commands.trigger(SpawnSoil {
                position: Vec2::new(x as f32 * 300.0, y as f32 * 300.0),
            })
        }
    }
}
