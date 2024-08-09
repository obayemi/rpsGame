use bevy::{
    app::{App, Plugin, Startup},
    core_pipeline::{bloom::BloomSettings, core_2d::Camera2dBundle, tonemapping::Tonemapping},
    ecs::system::Commands,
    render::camera::Camera,
};
use bevy_trauma_shake::Shake;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true, // required for bloom
                ..Default::default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..Default::default()
        },
        BloomSettings::NATURAL,
        Shake::default(),
    ));
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
    }
}
