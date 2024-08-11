use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

impl AnimationIndices {
    pub const fn from_frames(frames: usize) -> Self {
        Self {
            first: 0,
            last: frames - 1,
        }
    }

    pub const fn advance(&self, index: usize) -> usize {
        if index >= self.last {
            self.first
        } else {
            index + 1
        }
    }
}

pub fn animate_sprites(
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

#[derive(Component, Reflect, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

impl AnimationTimer {
    pub fn repeating(duration: f32) -> Self {
        Self(Timer::from_seconds(duration, TimerMode::Repeating))
    }
}

#[derive(Bundle)]
pub struct AnimatableSpriteBundle {
    pub sprite: SpriteBundle,
    pub texture_atlas: TextureAtlas,
    pub sprite_indices: AnimationIndices,
    pub timer: AnimationTimer,
}

impl AnimatableSpriteBundle {
    pub fn new(
        position: Vec3,
        scale: Vec3,
        texture: Handle<Image>,
        layout: Handle<TextureAtlasLayout>,
        indices: AnimationIndices,
        frame_time: f32,
    ) -> Self {
        Self {
            sprite: SpriteBundle {
                transform: Transform::from_xyz(position.x, position.y, position.z)
                    .with_scale(scale),
                texture,
                ..default()
            },
            texture_atlas: TextureAtlas {
                layout,
                index: indices.first,
            },
            sprite_indices: indices,
            timer: AnimationTimer::repeating(frame_time),
        }
    }
}
