use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<Health>();
    app.observe(damage);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Health {
    pub max: f32,
    pub current: f32,
}

impl Health {
    pub fn full(max: f32) -> Self {
        Self { max, current: max }
    }
}

#[derive(Event)]
pub struct Damage {
    pub amount: f32,
}

fn damage(
    trigger: Trigger<Damage>,
    mut health: Query<(Entity, &mut Health)>,
    mut commands: Commands,
) {
    if let Ok((entity, mut health)) = health.get_mut(trigger.entity()) {
        health.current -= trigger.event().amount;
        if health.current <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
