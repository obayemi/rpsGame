use std::time::Duration;

use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::Assets,
    color::Color,
    core::Name,
    ecs::system::SystemId,
    input::ButtonInput,
    math::{IVec3, Vec3},
    prelude::{
        info, Commands, Component, Deref, DerefMut, Entity, FromWorld, KeyCode, Mesh, Query,
        Rectangle, Res, ResMut, Resource, Transform, With, World,
    },
    reflect::Reflect,
    sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2dHandle},
    time::Timer,
};
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween};

use crate::{
    animations::AnimatableSpriteBundle,
    entity_gc::EntityLifetime,
    hand::{Hand, HandAnimations, HandBundle},
    movement::Velocity,
};

#[derive(Reflect)]
enum CanonState {
    Idle,
    Firing,
}

#[derive(Component, Reflect)]
struct HandCannon {
    fire_rate: Timer,
}

impl HandCannon {
    fn new(fire_rate: f32) -> Self {
        let mut timer = Timer::new(
            Duration::from_millis(fire_rate as u64 / 1000),
            bevy::time::TimerMode::Repeating,
        );
        Self { fire_rate: timer }
    }
}

#[derive(Component, Reflect, Hash, PartialEq, Eq, Copy, Clone)]
enum HandCannonState {
    Idle,
    InMotion,
}

#[derive(Component, Reflect, Deref, DerefMut)]
struct HandCannonTimer(Timer);

fn spawn_hand_cannon(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mesh = Mesh2dHandle(meshes.add(Rectangle::new(50.0, 100.0)));
    let color = Color::srgb(0.8, 0.5, 0.5);
    commands.spawn((
        MaterialMesh2dBundle {
            mesh,
            material: materials.add(color),
            transform: Transform::from_xyz(
                // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
                0.0, 0.0, 0.0,
            ),
            ..Default::default()
        },
        Name::new("Hand cannon"),
        HandCannonState::Idle,
        HandCannon::new(1.0),
        // HandCannonTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
    ));
}

fn direction_from_input(input: Res<ButtonInput<KeyCode>>) -> IVec3 {
    let right = i32::from(input.pressed(KeyCode::ArrowRight));
    let left = i32::from(input.pressed(KeyCode::ArrowLeft));
    let up = i32::from(input.pressed(KeyCode::ArrowUp));
    let down = i32::from(input.pressed(KeyCode::ArrowDown));

    IVec3::new(right - left, up - down, 0)
}

fn clear_movement_state(mut query: Query<&mut HandCannonState>) {
    let mut state = query.single_mut();
    *state = HandCannonState::Idle;
}

const MOVE_DISTANCE: f32 = 100.0;
const MOVE_SPEED: u64 = 100;
fn move_cannon(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut HandCannonState, &Transform), With<HandCannon>>,
    mut commands: Commands,
    clear_movement_state: Option<Res<ClearMovementSystemId>>,
) {
    if clear_movement_state.is_none() {
        return;
    }
    if let Ok((entity, mut cannonState, transform)) = query.get_single_mut() {
        if *cannonState == HandCannonState::InMotion {
            return;
        }
        let direction = direction_from_input(input);
        if direction.length_squared() != 0 {
            *cannonState = HandCannonState::InMotion;
            let tween = Tween::new(
                EaseFunction::ExponentialInOut,
                Duration::from_millis(MOVE_SPEED),
                TransformPositionLens {
                    start: transform.translation,
                    end: transform.translation + (direction.as_vec3() * MOVE_DISTANCE),
                },
            )
            .with_completed_system(clear_movement_state.unwrap().0);
            commands.entity(entity).insert(Animator::new(tween));
        };
    }
}

const FIRE_AMOUNT: u32 = 1;
const FIRE_SPREAD: f32 = 40.0;
const FIRE_RATE: f32 = 0.5;

fn fire_cannon(
    query: Query<(Entity, &Transform), With<HandCannon>>,
    hand_animations: Res<HandAnimations>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    if input.pressed(KeyCode::Space) {
        let (_, transform) = query.single();
        for i in 0..FIRE_AMOUNT.pow(2) {
            let hand = Hand::random();
            let texture = hand_animations.get(hand);
            let layout = hand_animations.layout();
            let indices = hand_animations.indices();

            let pos = Vec3::new(
                FIRE_SPREAD * ((i / FIRE_AMOUNT) as f32 - ((FIRE_AMOUNT - 1) as f32 / 2.0)),
                FIRE_SPREAD * ((i % FIRE_AMOUNT) as f32 - ((FIRE_AMOUNT - 1) as f32 / 2.0)),
                0.0,
            );
            commands.spawn((
                Name::new("Hand"),
                EntityLifetime::new(5.),
                HandBundle {
                    hand,
                    sprite: AnimatableSpriteBundle::new(
                        transform.translation + pos,
                        Vec3::splat(6.0),
                        texture,
                        layout,
                        indices,
                        0.25,
                    ),
                },
                Velocity::new(0.0, 1000.0, 0.0),
            ));
        }
    }
}

#[derive(Resource, Deref, Debug)]
struct ClearMovementSystemId(SystemId);
impl FromWorld for ClearMovementSystemId {
    fn from_world(world: &mut World) -> Self {
        Self(world.register_system(clear_movement_state))
    }
}

pub struct HandCannonPlugin;

impl Plugin for HandCannonPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HandCannonState>()
            .init_resource::<ClearMovementSystemId>()
            .add_systems(Update, (move_cannon, fire_cannon))
            .add_systems(Startup, spawn_hand_cannon);
    }
}
