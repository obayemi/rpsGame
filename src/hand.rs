use crate::{
    animations::{AnimatableSpriteBundle, AnimationIndices},
    particles::HanabiThing,
};
use bevy::{
    app::{App, Plugin, Update},
    asset::{AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        system::{Commands, Resource},
    },
    input::ButtonInput,
    math::UVec2,
    prelude::{Bundle, Changed, Entity, EventWriter, FromWorld, Image, KeyCode, Query, Res, World},
    reflect::Reflect,
    sprite::TextureAtlasLayout,
};
use bevy_hanabi::{CompiledParticleEffect, EffectSpawner};
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

/// assumes that all hand animations are 4 frames spritesheets of 32/32 pixels
#[derive(Resource, Reflect)]
pub struct HandAnimations {
    rock: Handle<Image>,
    paper: Handle<Image>,
    scissors: Handle<Image>,
    atlas_layout: Handle<TextureAtlasLayout>,
    indices: AnimationIndices,
}

impl HandAnimations {
    pub fn get(&self, hand: Hand) -> Handle<Image> {
        use Hand::*;
        match hand {
            Rock => self.rock.clone(),
            Paper => self.paper.clone(),
            Scissors => self.scissors.clone(),
        }
    }

    pub fn layout(&self) -> Handle<TextureAtlasLayout> {
        self.atlas_layout.clone()
    }

    pub fn indices(&self) -> AnimationIndices {
        self.indices.clone()
    }
}

impl FromWorld for HandAnimations {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let rock_texture: Handle<Image> = asset_server.load("hands/rock.png");
        let paper_texture: Handle<Image> = asset_server.load("hands/paper.png");
        let scissors_texture: Handle<Image> = asset_server.load("hands/scissors.png");

        let mut texture_atlas_layout = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let indices = AnimationIndices::from_frames(4);
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 4, 1, None, None);
        let atlas_layout = texture_atlas_layout.add(layout);

        Self {
            rock: rock_texture,
            paper: paper_texture,
            scissors: scissors_texture,
            atlas_layout,
            indices,
        }
    }
}

fn sync_hand_animation(
    mut query: Query<(&Hand, &mut Handle<Image>), Changed<Hand>>,
    animations: Res<HandAnimations>,
) {
    for (hand, mut texture) in &mut query.iter_mut() {
        *texture = animations.get(*hand).clone();
    }
}

fn change_hand(
    mut query: Query<(Entity, Option<&mut EffectSpawner>, &mut Hand)>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    hana: Res<HanabiThing>,
    mut trauma: EventWriter<TraumaEvent>,
) {
    if input.just_pressed(KeyCode::KeyA) {
        trauma.send(0.3.into());
        for (entity, effects, mut hand) in &mut query.iter_mut() {
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
