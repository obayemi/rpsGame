use crate::{
    animations::{AnimatableSpriteBundle, AnimationIndices, AnimationTimer},
    particles::HanabiThing,
};
use bevy::{
    app::{App, Plugin, Startup, Update},
    ecs::{
        component::Component,
        system::{Commands, Resource},
    },
    prelude::*,
    reflect::Reflect,
    utils::HashMap,
};
use bevy_hanabi::{CompiledParticleEffect, EffectSpawner, ParticleEffect};
use bevy_trauma_shake::TraumaEvent;
use rand::seq::SliceRandom;

pub struct HandPlugin;

impl Plugin for HandPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Hand>()
            .init_resource::<HandAnimations>()
            // .add_systems(Startup, spawn_hand)
            .add_systems(Update, (change_hand, sync_hand_animation));
    }
}

#[derive(Component, Reflect, Debug, Eq, Hash, PartialEq, Copy, Clone)]
pub enum Hand {
    Rock,
    Paper,
    Scissors,
}

impl Hand {
    pub fn random() -> Self {
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

#[derive(Resource)]
pub struct HandAnimations {
    map: HashMap<Hand, (Handle<Image>, Handle<TextureAtlasLayout>, AnimationIndices)>,
}

impl HandAnimations {
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
        }
    }

    pub fn get(&self, hand: Hand) -> (Handle<Image>, Handle<TextureAtlasLayout>, AnimationIndices) {
        self.map[&hand].clone()
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

fn sync_hand_animation(
    mut query: Query<(&Hand, &mut Handle<Image>), Changed<Hand>>,
    animations: Res<HandAnimations>,
) {
    for (hand, mut texture) in &mut query.iter_mut() {
        *texture = animations.map[hand].0.clone();
    }
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
                    hana.effect().with_z_layer_2d(Some(-0.1)),
                    CompiledParticleEffect::default(),
                ));
            }
            *hand = hand.cycle();
        }
    }
}

#[derive(Bundle)]
pub struct HandBundle {
    pub hand: Hand,
    pub sprite: AnimatableSpriteBundle,
}

fn spawn_hand(mut commands: Commands, hand_animations: Res<HandAnimations>) {
    // let hand = Hand::random();
    // let (texture, layout, indices) = &hand_animations.map[&hand];
    //
    // commands.spawn((
    //     Name::new("Hand"),
    //     HandBundle {
    //         hand,
    //         sprite: AnimatableSpriteBundle::new(
    //             Vec3::new(-100.0, 0.0, 0.0),
    //             Vec3::splat(6.0),
    //             texture.clone(),
    //             layout.clone(),
    //             indices.clone(),
    //             0.25,
    //         ),
    //     },
    // ));
    //
    // let hand = Hand::random();
    // commands.spawn((
    //     Name::new("Other Hand"),
    //     HandBundle {
    //         hand,
    //         sprite: AnimatableSpriteBundle::new(
    //             Vec3::new(100.0, 0.0, 0.0),
    //             Vec3::splat(6.0),
    //             texture.clone(),
    //             layout.clone(),
    //             indices.clone(),
    //             0.25,
    //         ),
    //     },
    // ));
}
