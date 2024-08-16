use bevy::prelude::*;
use bevy_hanabi::velocity;

#[derive(Component, Reflect)]
pub struct Velocity(Vec3);
impl Velocity {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3::new(x, y, z))
    }
    pub const fn from_vec3(v: Vec3) -> Self {
        Self(v)
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
        // (count) += 1;
        // info!("moved {count} items");
    })
}

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Velocity>()
            .add_systems(Update, move_things);
    }
}
