use bevy::{
    app::{App, Update},
    math::Vec3,
    prelude::{Component, Plugin, Query, Res, Transform},
    reflect::Reflect,
    time::Time,
};

#[derive(Component, Reflect)]
pub struct Velocity(Vec3);
impl Velocity {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3::new(x, y, z))
    }
}

impl From<Vec3> for Velocity {
    fn from(v: Vec3) -> Self {
        Self(v)
    }
}

pub struct MovementPlugin;

fn move_things(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    query.iter_mut().for_each(move |(velocity, mut transform)| {
        transform.translation += velocity.0 * time.delta_seconds();
    });
}

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Velocity>()
            .add_systems(Update, move_things);
    }
}
