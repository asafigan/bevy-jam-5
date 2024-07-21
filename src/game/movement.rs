//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/latest/examples/movement/physics_in_fixed_timestep.rs).

use std::time::Duration;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::AppSet;

use super::ghost::{GhostSet, GhostSpawner};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.register_type::<(MovementController, DashIntent)>();
    app.init_resource::<DashIntent>();
    app.add_systems(
        Update,
        (record_movement_controller, record_dash_intent).in_set(AppSet::RecordInput),
    );

    // Apply movement based on controls.
    app.register_type::<(MovementSettings, WrapWithinWindow, DashSettings, Dash)>();
    app.add_systems(
        Update,
        (
            start_dash,
            apply_movement,
            apply_dash.after(GhostSet::Update),
            stop_dash,
            wrap_within_window,
        )
            .chain()
            .in_set(AppSet::Update),
    );
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementController(pub Vec2);

impl MovementController {
    pub fn direction(&self) -> Option<Dir2> {
        self.0.try_into().ok()
    }
}

fn record_movement_controller(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    // Normalize so that diagonal movement has the same speed as
    // horizontal and vertical movement.
    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.0 = intent;
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementSettings {
    /// Since Bevy's default 2D camera setup is scaled such that
    /// one unit is one pixel, you can think of this as
    /// "How many pixels per second should the player move?"
    /// Note that physics engines may use different unit/pixel ratios.
    pub max_speed: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DashSettings {
    /// The input window that a dash is reconized.
    /// For example, if the player is in the middle of another action when intenting to dash,
    /// the intent will be delayed up to this window.
    pub intent_window: Duration,
    pub distance: f32,
    pub time: Duration,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Dash {
    pub start_time: Duration,
    pub direction: Dir2,
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<
        (&MovementController, &MovementSettings, &mut Transform),
        Without<Dash>,
    >,
) {
    for (controller, movement, mut transform) in &mut movement_query {
        let velocity = movement.max_speed * controller.0;
        transform.translation += velocity.extend(0.0) * time.delta_seconds();
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct DashIntent {
    pub at_time: Option<Duration>,
}

fn record_dash_intent(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut intent: ResMut<DashIntent>,
) {
    if input.just_pressed(KeyCode::Space) {
        intent.at_time = Some(time.elapsed());
    }
}

fn start_dash(
    time: Res<Time>,
    dash_settings_query: Query<(Entity, &DashSettings, &MovementController), Without<Dash>>,
    dash_intent: Res<DashIntent>,
    mut commands: Commands,
) {
    let Some(at_time) = dash_intent.at_time else {
        return;
    };
    let dash_window_end = time.elapsed();
    let dash_window_start = time.elapsed() - time.delta();
    for (entity, settings, movement_controller) in &dash_settings_query {
        if let Some(direction) = movement_controller.direction() {
            if dash_window_end - settings.intent_window <= at_time {
                commands.entity(entity).insert((
                    Dash {
                        start_time: dash_window_start,
                        direction,
                    },
                    GhostSpawner {
                        timer: Timer::new(Duration::from_millis(50), TimerMode::Repeating),
                        ghost_duration: Duration::from_millis(250),
                    },
                ));
            }
        }
    }
}

fn apply_dash(time: Res<Time>, mut dash_query: Query<(&Dash, &DashSettings, &mut Transform)>) {
    let previous_time = time.elapsed() - time.delta();
    for (dash, dash_settings, mut transform) in &mut dash_query {
        let already_done = (previous_time - dash.start_time).max(Duration::ZERO);
        let todo = ((already_done + time.delta()).min(dash_settings.time) - already_done)
            .max(Duration::ZERO);
        let delta_distance =
            (todo.as_secs_f32() / dash_settings.time.as_secs_f32()) * dash_settings.distance;
        let displacement = dash.direction * delta_distance;

        transform.translation += displacement.extend(0.0);
    }
}

fn stop_dash(
    time: Res<Time>,
    dash_query: Query<(Entity, &Dash, &DashSettings)>,
    mut commands: Commands,
) {
    for (entity, dash, dash_settings) in &dash_query {
        let since_start = time.elapsed() - dash.start_time;
        if since_start > dash_settings.time {
            commands
                .entity(entity)
                .remove::<Dash>()
                .remove::<GhostSpawner>();
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct WrapWithinWindow;

fn wrap_within_window(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut wrap_query: Query<&mut Transform, With<WrapWithinWindow>>,
) {
    let size = window_query.single().size() + 256.0;
    let half_size = size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let wrapped = (position + half_size).rem_euclid(size) - half_size;
        transform.translation = wrapped.extend(transform.translation.z);
    }
}
