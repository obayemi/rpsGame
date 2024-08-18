#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_precision_loss)]

mod animations;
mod camera;
#[cfg(feature = "debug")]
mod debug;
mod entity_gc;
mod hand;
mod hand_cannon;
mod movement;
mod particles;

use bevy::{
    app::{App, AppExit, Update},
    input::{keyboard::KeyCode, ButtonInput},
    prelude::{EventWriter, Image, ImagePlugin, PluginGroup, Res},
    DefaultPlugins,
};
use bevy_framepace::FramepacePlugin;
use bevy_tweening::TweeningPlugin;
use movement::MovementPlugin;
use particles::ParticlesPlugin;

use animations::AnimationsPlugin;
use camera::CameraPlugin;
#[cfg(feature = "debug")]
use debug::DebugPlugin;
use entity_gc::EntityGcPlugin;
use hand::HandPlugin;
use hand_cannon::HandCannonPlugin;

fn ui_things(input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
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
            TweeningPlugin,
            FramepacePlugin,
            #[cfg(feature = "debug")]
            DebugPlugin,
            ParticlesPlugin,
            MovementPlugin,
            HandCannonPlugin,
            AnimationsPlugin,
            EntityGcPlugin,
        ))
        .add_systems(Update, ui_things)
        .run();
}
