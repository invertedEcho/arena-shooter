use bevy::prelude::*;
use bevy_hanabi::prelude::*;

#[derive(Resource)]
pub struct BulletImpactEffectHandle(Option<Handle<EffectAsset>>);

#[derive(Event)]
pub struct SpawnBulletImpactEffectEvent {
    pub spawn_location: Vec3,
}

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_bullet_effect_handle)
            .add_systems(Update, spawn_bullet_impact_particle)
            .add_event::<SpawnBulletImpactEffectEvent>()
            .insert_resource(BulletImpactEffectHandle(None));
    }
}

fn setup_bullet_effect_handle(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    // Define a color gradient from red to transparent black
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1., 1., 1., 1.));
    gradient.add_key(1.0, Vec4::splat(0.));

    // Create a new expression module
    let mut module = Module::default();

    // On spawn, randomly initialize the position of the particle
    // to be over the surface of a sphere of radius with x units.
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(0.1),
        dimension: ShapeDimension::Volume,
    };

    // Also initialize a radial initial velocity to 6 units/sec
    // away from the (same) sphere center.
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(10.),
    };

    // Initialize the total lifetime of the particle, that is
    // the time for which it's simulated and rendered. This modifier
    // is almost always required, otherwise the particles won't show.
    let lifetime = module.lit(3.0);
    let init_lifetime =
        SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let size_attribute = SetAttributeModifier {
        attribute: Attribute::SIZE,
        value: module.lit(0.3),
    };
    info!("size attribute: {:?}", size_attribute);

    // Every frame, add a gravity-like acceleration downward
    let accel = module.lit(Vec3::new(0., -3., 0.));
    let update_accel = AccelModifier::new(accel);

    // Create the effect asset
    let effect = EffectAsset::new(
        // Maximum number of particles alive at a time
        32768,
        SpawnerSettings::once(1.0.into()),
        // Move the expression module into the asset
        module,
    )
    .with_name("BulletImpact")
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .init(size_attribute)
    .update(update_accel)
    // Render the particles with a color gradient over their
    // lifetime. This maps the gradient key 0 to the particle spawn
    // time, and the gradient key 1 to the particle death (10s).
    .render(ColorOverLifetimeModifier {
        gradient,
        ..default()
    });

    // Insert into the asset system
    let effect_handle = effects.add(effect);
    commands.insert_resource(BulletImpactEffectHandle(Some(effect_handle)));
}

fn spawn_bullet_impact_particle(
    mut commands: Commands,
    mut event_reader: EventReader<SpawnBulletImpactEffectEvent>,
    bullet_impact_effect_resource: Res<BulletImpactEffectHandle>,
) {
    for event in event_reader.read() {
        let Some(ref bullet_impact_effect_handle) =
            bullet_impact_effect_resource.0
        else {
            warn!(
                "Bullet impact effect spawn requested but effect handle does not exist!"
            );
            continue;
        };
        // info!(
        //     "Spawning bullet impact particle at: {}",
        //     event.spawn_location
        // );
        // commands.spawn((
        //     ParticleEffect::new(bullet_impact_effect_handle.clone()),
        //     Transform::from_translation(event.spawn_location),
        // ));
    }
}
