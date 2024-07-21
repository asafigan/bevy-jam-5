//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use std::time::Duration;

use bevy::prelude::*;

use super::{audio::sfx::PlaySfx, movement::Dash};
use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    app.register_type::<(PlayerAnimation, TranslationHistory)>();
    app.add_systems(
        PreUpdate,
        (
            remove_translation_history,
            add_translation_history,
            record_translation_history,
        )
            .chain(),
    );
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSet::TickTimers),
            (
                update_animation_movement,
                update_animation_atlas,
                trigger_step_sfx,
            )
                .chain()
                .in_set(AppSet::Update),
        ),
    );
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct TranslationHistory {
    pub previous: Vec3,
    pub delta: Vec3,
}

impl TranslationHistory {
    fn update(&mut self, new: Vec3) {
        self.delta = new - self.previous;
        self.previous = new;
    }
}

fn record_translation_history(mut transform_query: Query<(&Transform, &mut TranslationHistory)>) {
    for (transform, mut previous_translation) in &mut transform_query {
        previous_translation.update(transform.translation);
    }
}

fn add_translation_history(
    transform_query: Query<(Entity, &Transform), Without<TranslationHistory>>,
    mut commands: Commands,
) {
    for (entity, transform) in &transform_query {
        commands.entity(entity).insert(TranslationHistory {
            previous: transform.translation,
            delta: default(),
        });
    }
}

fn remove_translation_history(
    transform_query: Query<Entity, (With<TranslationHistory>, Without<Transform>)>,
    mut commands: Commands,
) {
    for entity in &transform_query {
        commands.entity(entity).remove::<TranslationHistory>();
    }
}

/// Update the sprite direction and animation state (idling/walking).
fn update_animation_movement(
    mut player_query: Query<(
        &TranslationHistory,
        &mut Sprite,
        &mut PlayerAnimation,
        Option<&Dash>,
    )>,
) {
    for (history, mut sprite, mut animation, dash) in &mut player_query {
        let dx = history.delta.x;
        if dx != 0.0 {
            sprite.flip_x = dx < 0.0;
        }

        let animation_state = if dash.is_some() {
            PlayerAnimationState::Dashing
        } else if history.delta.truncate() == Vec2::ZERO {
            PlayerAnimationState::Idling
        } else {
            PlayerAnimationState::Walking
        };
        animation.update_state(animation_state);
    }
}

/// Update the animation timer.
fn update_animation_timer(time: Res<Time>, mut query: Query<&mut PlayerAnimation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

/// Update the texture atlas to reflect changes in the animation.
fn update_animation_atlas(mut query: Query<(&PlayerAnimation, &mut TextureAtlas)>) {
    for (animation, mut atlas) in &mut query {
        if animation.changed() {
            atlas.index = animation.get_atlas_index();
        }
    }
}

/// If the player is moving, play a step sound effect synchronized with the animation.
fn trigger_step_sfx(mut commands: Commands, mut step_query: Query<&PlayerAnimation>) {
    for animation in &mut step_query {
        if animation.state == PlayerAnimationState::Walking
            && animation.changed()
            && (animation.frame == 2 || animation.frame == 5)
        {
            commands.trigger(PlaySfx::RandomStep);
        }
    }
}

/// Component that tracks player's animation state.
/// It is tightly bound to the texture atlas we use.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerAnimation {
    timer: Timer,
    frame: usize,
    state: PlayerAnimationState,
}

#[derive(Reflect, PartialEq)]
pub enum PlayerAnimationState {
    Idling,
    Walking,
    Dashing,
}

impl PlayerAnimation {
    /// The number of idle frames.
    const IDLE_FRAMES: usize = 2;
    /// The duration of each idle frame.
    const IDLE_INTERVAL: Duration = Duration::from_millis(500);

    fn idling() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Idling,
        }
    }

    /// The number of walking frames.
    const WALKING_FRAMES: usize = 6;
    /// The duration of each walking frame.
    const WALKING_INTERVAL: Duration = Duration::from_millis(50);

    fn walking() -> Self {
        Self {
            timer: Self::walking_timer(),
            frame: 0,
            state: PlayerAnimationState::Walking,
        }
    }

    fn dashing() -> Self {
        Self {
            timer: Self::dashing_timer(),
            frame: 0,
            state: PlayerAnimationState::Dashing,
        }
    }

    fn walking_timer() -> Timer {
        Timer::new(Self::WALKING_INTERVAL, TimerMode::Repeating)
    }

    fn dashing_timer() -> Timer {
        let mut timer = Self::walking_timer();
        timer.pause();
        timer
    }

    fn to_walking(&mut self) {
        match self.state {
            PlayerAnimationState::Idling => *self = Self::walking(),
            PlayerAnimationState::Walking => {}
            PlayerAnimationState::Dashing => {
                self.timer.unpause();
                self.state = PlayerAnimationState::Walking;
            }
        }
    }

    fn to_dashing(&mut self) {
        match self.state {
            PlayerAnimationState::Idling => *self = Self::dashing(),
            PlayerAnimationState::Walking => {
                self.timer.pause();
                self.state = PlayerAnimationState::Dashing;
            }
            PlayerAnimationState::Dashing => {}
        }
    }

    pub fn new() -> Self {
        Self::idling()
    }

    /// Update animation timers.
    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.finished() {
            return;
        }
        self.frame = (self.frame + 1)
            % match self.state {
                PlayerAnimationState::Idling => Self::IDLE_FRAMES,
                PlayerAnimationState::Walking => Self::WALKING_FRAMES,
                PlayerAnimationState::Dashing => Self::WALKING_FRAMES,
            };
    }

    /// Update animation state if it changes.
    pub fn update_state(&mut self, state: PlayerAnimationState) {
        if self.state != state {
            match state {
                PlayerAnimationState::Idling => *self = Self::idling(),
                PlayerAnimationState::Walking => self.to_walking(),
                PlayerAnimationState::Dashing => self.to_dashing(),
            }
        }
    }

    /// Whether animation changed this tick.
    pub fn changed(&self) -> bool {
        self.timer.finished()
    }

    /// Return sprite index in the atlas.
    pub fn get_atlas_index(&self) -> usize {
        match self.state {
            PlayerAnimationState::Idling => self.frame,
            PlayerAnimationState::Walking | PlayerAnimationState::Dashing => 6 + self.frame,
        }
    }
}
