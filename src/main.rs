#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::needless_pass_by_value)]

use std::time::Duration;

use bevy::{
    core_pipeline::bloom::BloomSettings,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
};
use bevy_hanabi::prelude::*;
#[cfg(feature = "debug")]
use bevy_inspector_egui::{quick::WorldInspectorPlugin, DefaultInspectorConfigPlugin};
use bevy_trauma_shake::{Shake, TraumaEvent, TraumaPlugin};
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween, TweeningPlugin};
use rand::seq::SliceRandom;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            // transform: Transform::from_xyz(0.0, 0.0, 0.0),
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            tonemapping: bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            ..default()
        },
        BloomSettings::NATURAL,
        // BloomSettings {
        //     high_pass_frequency: 0.01,
        //     intensity: 0.2,
        //     low_frequency_boost: 1.0,
        //     ..Default::default()
        // }, // 3. Enable bloom for the camera
        Shake::default(),
    ));
}

#[derive(Component, Clone, Debug)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

impl AnimationIndices {
    const fn from_frames(frames: usize) -> Self {
        Self {
            first: 0,
            last: frames - 1,
        }
    }

    const fn advance(&self, index: usize) -> usize {
        if index >= self.last {
            self.first
        } else {
            index + 1
        }
    }
}

#[derive(Component, Reflect, Debug, Eq, Hash, PartialEq, Copy, Clone)]
enum Hand {
    Rock,
    Paper,
    Scissors,
}

impl Hand {
    fn random() -> Self {
        use Hand::{Paper, Rock, Scissors};
        *[Rock, Paper, Scissors]
            .choose(&mut rand::thread_rng())
            .unwrap()
    }

    const fn cycle(self) -> Self {
        use Hand::{Paper, Rock, Scissors};
        match self {
            Rock => Paper,
            Paper => Scissors,
            Scissors => Rock,
        }
    }
}

#[derive(Component, Reflect, Deref, DerefMut)]
struct AnimationTimer(Timer);

impl AnimationTimer {
    fn repeating(duration: f32) -> Self {
        Self(Timer::from_seconds(duration, TimerMode::Repeating))
    }
}

#[derive(Resource)]
struct HandAnimations {
    map: HashMap<Hand, (Handle<Image>, Handle<TextureAtlasLayout>, AnimationIndices)>,
}

impl HandAnimations {
    fn new() -> Self {
        Self {
            map: HashMap::default(),
        }
    }
}

impl FromWorld for HandAnimations {
    fn from_world(world: &mut World) -> Self {
        let mut animations = Self::new();
        let asset_server = world.resource::<AssetServer>();

        let rock_texture: Handle<Image> = asset_server.load("hands/rock.png");
        let paper_texture: Handle<Image> = asset_server.load("hands/paper.png");
        let scissors_texture: Handle<Image> = asset_server.load("hands/scissors.png");

        let mut texture_atlas_layout = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let indices = AnimationIndices::from_frames(4);
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 4, 1, None, None);
        let atlas_layout = texture_atlas_layout.add(layout);

        animations.map.insert(
            Hand::Rock,
            (rock_texture, atlas_layout.clone(), indices.clone()),
        );
        animations.map.insert(
            Hand::Paper,
            (paper_texture, atlas_layout.clone(), indices.clone()),
        );
        animations
            .map
            .insert(Hand::Scissors, (scissors_texture, atlas_layout, indices));

        animations
    }
}

fn spawn_hand(mut commands: Commands, hand_animations: Res<HandAnimations>) {
    let hand = Hand::random();
    let (texture, layout, indices) = &hand_animations.map[&hand];

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(-100.0, 0.0, 0.0).with_scale(Vec3::splat(6.0)),
            texture: texture.clone(),
            ..default()
        },
        TextureAtlas {
            layout: layout.clone(),
            index: indices.first,
        },
        indices.clone(),
        AnimationTimer::repeating(0.25),
        Name::new("Hand"),
        Bouncy::new(0.5),
        hand,
    ));

    let hand = Hand::random();
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(100.0, 0.0, 0.0).with_scale(Vec3::splat(6.0)),
            texture: texture.clone(),
            ..default()
        },
        TextureAtlas {
            layout: layout.clone(),
            index: indices.first,
        },
        indices.clone(),
        AnimationTimer::repeating(0.25),
        Name::new("Other Hand"),
        hand,
    ));
}

fn change_hand(
    mut query: Query<(
        Entity,
        &mut Handle<Image>,
        Option<&mut EffectSpawner>,
        &mut Hand,
    )>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    hana: Res<HanabiThing>,
    mut trauma: EventWriter<TraumaEvent>,
) {
    if input.just_pressed(KeyCode::KeyA) {
        trauma.send(0.3.into());
        for (entity, mut image, effects, mut hand) in &mut query.iter_mut() {
            if let Some(mut effects) = effects {
                effects.reset();
            } else {
                commands.entity(entity).insert((
                    ParticleEffect::new(hana.boom.clone()).with_z_layer_2d(Some(-0.1)),
                    CompiledParticleEffect::default(),
                ));
            }
            *hand = hand.cycle();
        }
    }
}

fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = indices.advance(atlas.index);
        }
    }
}

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

#[derive(Component, Deref, DerefMut)]
struct Bouncy(Timer);

impl Bouncy {
    fn new(duration: f32) -> Self {
        Self(Timer::from_seconds(duration, TimerMode::Repeating))
    }
}

fn bounce(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Bouncy)>,
) {
    for (entity, mut transform, mut timer) in &mut query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            println!("Bounce!");
        }
    }
}

fn sync_hand_animation(
    mut query: Query<(&Hand, &mut Handle<Image>), Changed<Hand>>,
    animations: Res<HandAnimations>,
) {
    for (hand, mut texture) in &mut query.iter_mut() {
        *texture = animations.map[hand].0.clone();
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
    if input.just_pressed(KeyCode::KeyW) {
        for (entity, transform) in &mut query.iter() {
            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
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
                EaseFunction::QuadraticInOut,
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

use clap::{ArgAction, Parser};
/// Thing
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, action = ArgAction::SetTrue)]
    debug: bool,
}

fn main() {
    let args = Args::parse();
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            // FrameTimeDiagnosticsPlugin,
            // LogDiagnosticsPlugin::default(),
            HanabiPlugin,
            TraumaPlugin,
            #[cfg(feature = "debug")]
            (WorldInspectorPlugin::new()),
            TweeningPlugin,
        ))
        .register_type::<Hand>()
        .register_type::<AnimationTimer>()
        .register_type::<Image>()
        .init_resource::<HandAnimations>()
        .init_resource::<HanabiThing>()
        .add_systems(Startup, setup_camera)
        .add_systems(Update, ui_things)
        .add_systems(Startup, spawn_hand)
        .add_systems(Startup, spawn_hand_cannon)
        .add_systems(Update, animate_sprites)
        .add_systems(Update, (change_hand, sync_hand_animation))
        .add_systems(Update, manual_fire)
        .run();
}
