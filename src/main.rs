#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::needless_pass_by_value)]

mod animations;
mod camera;
mod hand;

use bevy::{
    app::{App, Plugin, Startup, Update},
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
use bevy_hanabi::prelude::*;
#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_trauma_shake::TraumaPlugin;
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween, TweeningPlugin};
use std::time::Duration;

use animations::{animate_sprites, AnimationIndices, AnimationTimer};
use camera::CameraPlugin;
use hand::{Hand, HandPlugin};

// #[derive(component, Reflect)]
// struct Velocity(Vec3);

#[derive(Resource, Debug)]
struct HanabiThing {
    boom: Handle<EffectAsset>,
}

impl FromWorld for HanabiThing {
    fn from_world(world: &mut World) -> Self {
        let mut effects = world.resource_mut::<Assets<EffectAsset>>();

        // Define a color gradient from red to transparent black
        let mut gradient = Gradient::new();
        gradient.add_key(0.0, Vec4::new(0.686, 0.365, 0.40, 1.0));
        gradient.add_key(1.0, Vec4::splat(0.));

        let mut module = Module::default();

        let init_pos = SetPositionSphereModifier {
            center: module.lit(Vec3::new(0., 0., 1.)),
            radius: module.lit(50.),
            dimension: ShapeDimension::Surface,
        };

        let init_vel = SetVelocitySphereModifier {
            center: module.lit(Vec3::ZERO),
            speed: module.lit(500.),
        };

        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(0.25));

        let size = SetSizeModifier {
            size: CpuValue::Single(Vec2::splat(10.)),
        };

        // Create the effect asset
        let effect = EffectAsset::new(vec![32768], Spawner::once(100.0.into(), true), module)
            .with_name("MyEffect")
            .init(init_pos)
            .init(init_vel)
            .init(init_lifetime)
            .render(ColorOverLifetimeModifier { gradient })
            .render(size);

        Self {
            boom: effects.add(effect),
        }
    }
}

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
        HandCannon,
        // HandCannonTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
    ));
}

fn manual_fire(
    input: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Transform), With<HandCannon>>,
    mut commands: Commands,
) {
    let ease_function = EaseFunction::ExponentialInOut;
    if input.just_pressed(KeyCode::KeyW) {
        for (entity, transform) in &mut query.iter() {
            let tween = Tween::new(
                ease_function,
                Duration::from_millis(200),
                TransformPositionLens {
                    start: transform.translation,
                    end: transform.translation + Vec3::new(0.0, 100.0, 0.0),
                },
            );
            commands.entity(entity).insert(Animator::new(tween));
        }
    }
    if input.just_pressed(KeyCode::KeyS) {
        for (entity, transform) in &mut query.iter() {
            let tween = Tween::new(
                ease_function,
                Duration::from_millis(200),
                TransformPositionLens {
                    start: transform.translation,
                    end: transform.translation + Vec3::new(0.0, -100.0, 0.0),
                },
            );
            commands.entity(entity).insert(Animator::new(tween));
            // transform.translation.y += 100.0;
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

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            CameraPlugin,
            HandPlugin,
            HanabiPlugin,
            TraumaPlugin,
            #[cfg(feature = "debug")]
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Equal)),
            TweeningPlugin,
        ))
        .register_type::<AnimationTimer>()
        .register_type::<Image>()
        .init_resource::<HanabiThing>()
        .add_systems(Update, ui_things)
        .add_systems(Startup, spawn_hand_cannon)
        .add_systems(Update, animate_sprites)
        .add_systems(Update, manual_fire)
        .run();
}
