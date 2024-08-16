#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::needless_pass_by_value)]

mod animations;
mod camera;
mod debug;
mod hand;
mod movement;
mod particles;

use bevy::{
    app::{App, Plugin, Startup, Update},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::{
        component::Component,
        prelude::FromWorld,
        query::WorldQuery,
        system::{Commands, Resource},
    },
    input::{common_conditions::input_toggle_active, keyboard::KeyCode},
    math::{Vec3, Vec4},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    DefaultPlugins,
};
use bevy_framepace::FramepacePlugin;
use bevy_trauma_shake::TraumaPlugin;
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween, TweeningPlugin};
use movement::{MovementPlugin, Velocity};
use particles::ParticlesPlugin;
use std::time::Duration;

use animations::{animate_sprites, AnimatableSpriteBundle, AnimationIndices, AnimationTimer};
use camera::CameraPlugin;
use debug::DebugPlugin;
use hand::{Hand, HandAnimations, HandBundle, HandPlugin};

#[derive(Component, Reflect)]
struct HandCannon;

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
            ..default()
        },
        Name::new("Hand cannon"),
        HandCannon,
        // HandCannonTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
    ));
}

const MOVE_DISTANCE: f32 = 100.0;
const MOVE_SPEED: u64 = 100;
fn move_cannon(
    input: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Transform), With<HandCannon>>,
    mut commands: Commands,
) {
    let ease_function = EaseFunction::ExponentialInOut;
    if input.just_pressed(KeyCode::ArrowUp) {
        for (entity, transform) in &mut query.iter() {
            let tween = Tween::new(
                ease_function,
                Duration::from_millis(MOVE_SPEED),
                TransformPositionLens {
                    start: transform.translation,
                    end: transform.translation + Vec3::new(0.0, MOVE_DISTANCE, 0.0),
                },
            );
            commands.entity(entity).insert(Animator::new(tween));
        }
    }

    if input.just_pressed(KeyCode::ArrowDown) {
        for (entity, transform) in &mut query.iter() {
            let tween = Tween::new(
                ease_function,
                Duration::from_millis(MOVE_SPEED),
                TransformPositionLens {
                    start: transform.translation,
                    end: transform.translation + Vec3::new(0.0, -MOVE_DISTANCE, 0.0),
                },
            );
            commands.entity(entity).insert(Animator::new(tween));
        }
    }

    if input.just_pressed(KeyCode::ArrowLeft) {
        for (entity, transform) in &mut query.iter() {
            let tween = Tween::new(
                ease_function,
                Duration::from_millis(MOVE_SPEED),
                TransformPositionLens {
                    start: transform.translation,
                    end: transform.translation + Vec3::new(-MOVE_DISTANCE, 0.0, 0.0),
                },
            );
            commands.entity(entity).insert(Animator::new(tween));
        }
    }

    if input.just_pressed(KeyCode::ArrowRight) {
        for (entity, transform) in &mut query.iter() {
            let tween = Tween::new(
                ease_function,
                Duration::from_millis(MOVE_SPEED),
                TransformPositionLens {
                    start: transform.translation,
                    end: transform.translation + Vec3::new(MOVE_DISTANCE, 0.0, 0.0),
                },
            );
            commands.entity(entity).insert(Animator::new(tween));
            // transform.translation.y += 100.0;
        }
    }
}
const FIRE_AMOUNT: usize = 10;
const FIRE_SPREAD: f32 = 40.0;

fn fire_cannon(
    query: Query<(Entity, &Transform), With<HandCannon>>,
    hand_animations: Res<HandAnimations>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::Space) {
        let (_, transform) = query.single();
        for i in 0..FIRE_AMOUNT.pow(2) {
            let hand = Hand::random();
            let (texture, layout, indices) = hand_animations.get(hand);

            let pos = Vec3::new(
                FIRE_SPREAD * ((i / FIRE_AMOUNT) as f32 - ((FIRE_AMOUNT - 1) as f32 / 2.0)),
                FIRE_SPREAD * ((i % FIRE_AMOUNT) as f32 - ((FIRE_AMOUNT - 1) as f32 / 2.0)),
                0.0,
            );
            commands.spawn((
                Name::new("Hand"),
                HandBundle {
                    hand,
                    sprite: AnimatableSpriteBundle::new(
                        transform.translation + pos,
                        Vec3::splat(6.0),
                        texture.clone(),
                        layout.clone(),
                        indices.clone(),
                        0.25,
                    ),
                },
                Velocity::new(0.0, 1000.0, 0.0),
            ));
        }
    }
}

fn ui_things(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut exit: EventWriter<AppExit>,
) {
    if input.any_just_pressed([KeyCode::Escape, KeyCode::KeyQ]) {
        exit.send(AppExit::Success);
    }
}

/// Marker to find the container entity so we can show/hide the FPS counter
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            CameraPlugin,
            HandPlugin,
            TraumaPlugin,
            TweeningPlugin,
            FramepacePlugin,
            DebugPlugin,
            ParticlesPlugin,
            MovementPlugin,
        ))
        .register_type::<AnimationTimer>()
        .register_type::<Image>()
        .add_systems(Startup, spawn_hand_cannon)
        .add_systems(Update, animate_sprites)
        .add_systems(Update, (move_cannon, fire_cannon))
        .add_systems(Update, ui_things)
        .run();
}
