use bevy::prelude::*;
use bevy_hanabi::prelude::*;

#[derive(Resource, Debug)]
pub struct HanabiThing {
    boom: Handle<EffectAsset>,
}

impl HanabiThing {
    pub fn effect(&self) -> ParticleEffect {
        ParticleEffect::new(self.boom.clone())
    }
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

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin).init_resource::<HanabiThing>();
    }
}
