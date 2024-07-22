//! Game mechanics and content.

use bevy::prelude::*;

mod animation;
pub mod assets;
pub mod audio;
mod ghost;
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
    ));
}
