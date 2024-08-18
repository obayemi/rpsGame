use bevy::{
    app::{Plugin, Update},
    prelude::{Commands, Component, Deref, DerefMut, Entity, Query, Res},
    reflect::Reflect,
    time::{Time, Timer, TimerMode},
};

#[derive(Component, Reflect, Deref, DerefMut)]
pub struct EntityLifetime(Timer);
impl EntityLifetime {
    pub fn new(ttl: f32) -> Self {
        Self(Timer::from_seconds(ttl, TimerMode::Once))
    }
}

fn delete_expired_entities(
    mut query: Query<(Entity, &mut EntityLifetime)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    query.iter_mut().for_each(|(entity, mut lifetimer)| {
        lifetimer.tick(time.delta());
        if lifetimer.just_finished() {
            commands.entity(entity).despawn();
        }
    });
}

pub struct EntityGcPlugin;
impl Plugin for EntityGcPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<EntityLifetime>()
            .add_systems(Update, delete_expired_entities);
    }
}
