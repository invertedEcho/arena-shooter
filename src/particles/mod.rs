use std::f32::consts::PI;

use avian3d::math::{FRAC_1_SQRT_2, FRAC_PI_2};
use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::utils::random::{
    get_random_number_from_range_i32,
    get_random_number_from_range_i32_to_f32_with_step,
};

#[derive(Resource)]
pub struct BulletImpactEffectHandle(Option<Handle<EffectAsset>>);

#[derive(Event)]
pub struct SpawnBulletImpactEffectEvent {
    pub spawn_location: Vec3,
    pub rotate_towards_target: Vec3,
}

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_bullet_effect_handle)
            .add_systems(Update, handle_spawn_bullet_impact_effect)
            .add_event::<SpawnBulletImpactEffectEvent>()
            .insert_resource(BulletImpactEffectHandle(None));
    }
}

// TODO: rotate particles, e.g. if user facing towards X, set init_position.axis to X, if -X, set
// init_position.axis -X
fn setup_bullet_effect_handle(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let expression_writer = ExprWriter::new();

    // gradient colors
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1., 1., 1., 1.));
    gradient.add_key(1.0, Vec4::splat(0.));

    // where the particles are initially positioned
    let init_position = SetPositionCircleModifier {
        center: expression_writer.lit(Vec3::ZERO).expr(),
        axis: expression_writer.lit(Vec3::Z).expr(),
        radius: expression_writer.lit(0.1).expr(),
        dimension: ShapeDimension::Volume,
    };

    // particles "fly" away from center by speed of 3 units/sec
    let init_velocity = SetVelocityCircleModifier {
        center: expression_writer.lit(Vec3::ZERO).expr(),
        axis: expression_writer.lit(Vec3::Z).expr(),
        speed: expression_writer.lit(3.).expr(),
    };

    // how long the particles are visible
    let lifetime_handle = expression_writer.lit(0.2).expr();
    let init_lifetime =
        SetAttributeModifier::new(Attribute::LIFETIME, lifetime_handle);

    // the scale/size of the particles
    let size_attribute = SetAttributeModifier {
        attribute: Attribute::SIZE,
        value: expression_writer.lit(0.1).expr(),
    };

    // face particles towards camera
    let orient_modifier_billboard = OrientModifier {
        mode: OrientMode::FaceCameraPosition,
        rotation: None,
    };

    let repulsor_accel = expression_writer
        .add_property("repulsor_accel", Value::Scalar((-15.0).into()));
    // let repulsor_position = expression_writer
    //     .add_property("repulsor_position", Value::Vector(REPULSOR_POS.into()));
    let repulsor_accel = expression_writer.prop(repulsor_accel);
    let update_repulsor = ConformToSphereModifier {
        origin: expression_writer.lit(Vec3::ZERO).expr(),
        radius: expression_writer.lit(0.05).expr(),
        influence_dist: expression_writer.lit(0.05 * 10.).expr(),
        attraction_accel: repulsor_accel.expr(),
        max_attraction_speed: expression_writer.lit(10.).expr(),
        sticky_factor: None,
        shell_half_thickness: None,
    };

    let module = expression_writer.finish();
    let effect = EffectAsset::new(
        // Maximum number of particles alive at a time
        32768,
        SpawnerSettings::once(4.0.into()),
        // Move the expression module into the asset
        module,
    )
    .with_name("BulletImpact")
    .init(init_position)
    .init(init_velocity)
    .init(init_lifetime)
    .init(size_attribute)
    // .update(acceleration_modifier)
    // Render the particles with a color gradient over their
    // lifetime. This maps the gradient key 0 to the particle spawn
    // time, and the gradient key 1 to the particle death (10s).
    .render(ColorOverLifetimeModifier {
        gradient,
        ..default()
    })
    .update(update_repulsor)
    .render(orient_modifier_billboard);

    // Insert into the asset system
    let effect_handle = effects.add(effect);
    commands.insert_resource(BulletImpactEffectHandle(Some(effect_handle)));
}

fn handle_spawn_bullet_impact_effect(
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

        let random_z_rotation =
            get_random_number_from_range_i32_to_f32_with_step(0, 5, 0.1);
        let transform = Transform {
            translation: Vec3 {
                x: event.spawn_location.x,
                y: event.spawn_location.y,
                z: event.spawn_location.z + 0.2,
            },
            rotation: Quat::from_rotation_z(random_z_rotation as f32),
            ..default()
        };

        commands.spawn((
            ParticleEffect::new(bullet_impact_effect_handle.clone()),
            transform,
        ));
    }
}
