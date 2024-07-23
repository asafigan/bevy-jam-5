//! Game mechanics and content.

use bevy::prelude::*;

mod animation;
pub mod assets;
pub mod audio;
mod bullets;
pub mod collision_groups;
mod enemies;
mod ghost;
pub mod health;
mod movement;
mod plant;
pub mod spawn;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        audio::plugin,
        assets::plugin,
        ghost::plugin,
        movement::plugin,
        spawn::plugin,
        plant::plugin,
        enemies::plugin,
        bullets::plugin,
        health::plugin,
    ));
}
