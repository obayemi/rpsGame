#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::needless_pass_by_value)]

mod animations;
mod camera;
mod hand;

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
use bevy_hanabi::prelude::*;
#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_trauma_shake::TraumaPlugin;
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween, TweeningPlugin};
use iyes_perf_ui::{
    entries::PerfUiBundle,
    prelude::{PerfUiCompleteBundle, PerfUiEntryFPS, PerfUiRoot, PerfUiWidgetBar},
};
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
    if input.just_pressed(KeyCode::ArrowUp) {
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

    if input.just_pressed(KeyCode::ArrowDown) {
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

    if input.just_pressed(KeyCode::ArrowLeft) {
        for (entity, transform) in &mut query.iter() {
            let tween = Tween::new(
                ease_function,
                Duration::from_millis(200),
                TransformPositionLens {
                    start: transform.translation,
                    end: transform.translation + Vec3::new(-100.0, 0.0, 0.0),
                },
            );
            commands.entity(entity).insert(Animator::new(tween));
        }
    }

    if input.just_pressed(KeyCode::ArrowRight) {
        for (entity, transform) in &mut query.iter() {
            let tween = Tween::new(
                ease_function,
                Duration::from_millis(200),
                TransformPositionLens {
                    start: transform.translation,
                    end: transform.translation + Vec3::new(100.0, 0.0, 0.0),
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

/// Marker to find the container entity so we can show/hide the FPS counter
#[derive(Component)]
struct FpsRoot;

/// Marker to find the text entity so we can update it
#[derive(Component)]
struct FpsText;

fn setup_fps_counter(mut commands: Commands) {
    // create our UI root node
    // this is the wrapper/container for the text
    let root = commands
        .spawn((
            FpsRoot,
            NodeBundle {
                // give it a dark background for readability
                background_color: BackgroundColor(Color::BLACK.with_alpha(0.5)),
                // make it "always on top" by setting the Z index to maximum
                // we want it to be displayed over all other UI
                z_index: ZIndex::Global(i32::MAX),
                style: Style {
                    position_type: PositionType::Absolute,
                    // position it at the top-right corner
                    // 1% away from the top window edge
                    right: Val::Percent(1.),
                    top: Val::Percent(1.),
                    // set bottom/left to Auto, so it can be
                    // automatically sized depending on the text
                    bottom: Val::Auto,
                    left: Val::Auto,
                    // give it some padding for readability
                    padding: UiRect::all(Val::Px(4.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .id();
    // create our text
    let text_fps = commands
        .spawn((
            FpsText,
            TextBundle {
                // use two sections, so it is easy to update just the number
                text: Text::from_sections([
                    TextSection {
                        value: "FPS: ".into(),
                        style: TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            // if you want to use your game's font asset,
                            // uncomment this and provide the handle:
                            // font: my_font_handle
                            ..default()
                        },
                    },
                    TextSection {
                        value: " N/A".into(),
                        style: TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            // if you want to use your game's font asset,
                            // uncomment this and provide the handle:
                            // font: my_font_handle
                            ..default()
                        },
                    },
                ]),
                ..Default::default()
            },
        ))
        .id();
    commands.entity(root).push_children(&[text_fps]);
}

fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        // try to get a "smoothed" FPS value from Bevy
        if let Some(value) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            // Format the number as to leave space for 4 digits, just in case,
            // right-aligned and rounded. This helps readability when the
            // number changes rapidly.
            text.sections[1].value = format!("{value:>4.0}");

            // Let's make it extra fancy by changing the color of the
            // text according to the FPS value:
            text.sections[1].style.color = if value >= 120.0 {
                // Above 120 FPS, use green color
                Color::rgb(0.0, 1.0, 0.0)
            } else if value >= 60.0 {
                // Between 60-120 FPS, gradually transition from yellow to green
                Color::rgb((1.0 - (value - 60.0) / (120.0 - 60.0)) as f32, 1.0, 0.0)
            } else if value >= 30.0 {
                // Between 30-60 FPS, gradually transition from red to yellow
                Color::rgb(1.0, ((value - 30.0) / (60.0 - 30.0)) as f32, 0.0)
            } else {
                // Below 30 FPS, use red color
                Color::rgb(1.0, 0.0, 0.0)
            }
        } else {
            // display "N/A" if we can't get a FPS measurement
            // add an extra space to preserve alignment
            text.sections[1].value = " N/A".into();
            text.sections[1].style.color = Color::WHITE;
        }
    }
}

/// Toggle the FPS counter when pressing F12
fn fps_counter_showhide(
    mut q: Query<&mut Visibility, With<FpsRoot>>,
    kbd: Res<ButtonInput<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        let mut vis = q.single_mut();
        *vis = match *vis {
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Hidden,
        };
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
            FramepacePlugin,
            bevy_framepace::debug::DiagnosticsPlugin,
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .register_type::<AnimationTimer>()
        .register_type::<Image>()
        .init_resource::<HanabiThing>()
        .add_systems(Startup, setup_fps_counter)
        .add_systems(Update, (fps_text_update_system, fps_counter_showhide))
        .add_systems(Startup, spawn_hand_cannon)
        .add_systems(Update, animate_sprites)
        .add_systems(Update, manual_fire)
        .add_systems(Update, ui_things)
        .run();
}
