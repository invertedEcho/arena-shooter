use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::{player::Player, utils::random::get_random_number_from_range};

const BULLET_IMPACT_PARTICLE_LIFETIME: f32 = 0.1;
const BULLET_IMPACT_PARTICLE_VELOCITY: f32 = 3.0;

#[derive(Resource, Default)]
pub struct BulletImpactEffectHandle(Option<Handle<EffectAsset>>);

#[derive(Resource, Default)]
pub struct PlayerBulletHitEnemyImpactEffectHandle(Option<Handle<EffectAsset>>);

#[derive(Message)]
pub struct SpawnBulletImpactEffectMessage {
    pub spawn_location: Vec3,
    pub variant: BulletImpactEffectVariant,
}

pub enum BulletImpactEffectVariant {
    World,
    Enemy,
}

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                setup_bullet_effect_handle,
                setup_player_bullet_impact_enemy_handle,
            ),
        )
        .add_systems(Update, handle_spawn_bullet_impact_effect)
        .add_message::<SpawnBulletImpactEffectMessage>()
        .insert_resource(BulletImpactEffectHandle::default())
        .insert_resource(PlayerBulletHitEnemyImpactEffectHandle::default());
    }
}

fn setup_bullet_effect_handle(
    mut effects: ResMut<Assets<EffectAsset>>,
    mut bullet_impact_effect_resource: ResMut<BulletImpactEffectHandle>,
) {
    let expression_writer = ExprWriter::new();

    // gradient colors
    let mut gradient = bevy_hanabi::Gradient::new();
    gradient.add_key(0.0, Vec4::new(0.69, 0.69, 0.69, 1.));
    gradient.add_key(0.25, Vec4::new(1.0, 0.843, 0.0, 1.0));
    gradient.add_key(0.75, Vec4::new(0.678, 0.849, 0.902, 1.0));
    gradient.add_key(1.0, Vec4::new(1.0, 0.271, 0.0, 1.0));

    // where the origin of the particles is (randomly)
    let init_position_modifier = SetPositionCircleModifier {
        center: expression_writer.lit(Vec3::ZERO).expr(),
        axis: expression_writer.lit(Vec3::Z).expr(),
        radius: expression_writer.lit(0.1).expr(),
        dimension: ShapeDimension::Surface,
    };

    // particles "fly" away from center by speed of 3 units/sec
    let init_velocity = SetVelocityCircleModifier {
        center: expression_writer.lit(Vec3::ZERO).expr(),
        axis: expression_writer.lit(Vec3::Z).expr(),
        speed: expression_writer
            .lit(BULLET_IMPACT_PARTICLE_VELOCITY)
            .expr(),
    };

    // how long the particles are visible
    let lifetime_handle = expression_writer
        .lit(BULLET_IMPACT_PARTICLE_LIFETIME)
        .expr();
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

    let module = expression_writer.finish();
    let bullet_impact_effect =
        EffectAsset::new(1000, SpawnerSettings::once(4.0.into()), module)
            .with_name("BulletImpactStraight")
            .init(init_position_modifier)
            .init(init_velocity)
            .init(init_lifetime)
            .init(size_attribute)
            .render(orient_modifier_billboard)
            .render(ColorOverLifetimeModifier {
                gradient,
                ..default()
            });

    let effect_handle = effects.add(bullet_impact_effect);
    bullet_impact_effect_resource.0 = Some(effect_handle);
}

fn handle_spawn_bullet_impact_effect(
    mut commands: Commands,
    mut message_reader: MessageReader<SpawnBulletImpactEffectMessage>,
    bullet_impact_effect_resource: Res<BulletImpactEffectHandle>,
    bullet_impact_body_effect_resource: Res<
        PlayerBulletHitEnemyImpactEffectHandle,
    >,
    player_camera_transform_global: Single<&Transform, With<Player>>,
) {
    for message in message_reader.read() {
        let Some(bullet_impact_effect_handle) = (match message.variant {
            BulletImpactEffectVariant::World => {
                &bullet_impact_effect_resource.0
            }
            BulletImpactEffectVariant::Enemy => {
                &bullet_impact_body_effect_resource.0
            }
        }) else {
            continue;
        };

        let random_z_rotation = get_random_number_from_range(0..5) as f32;

        let rotation_z = Quat::from_rotation_z(random_z_rotation);
        let rotation_towards_player_perpendicular =
            player_camera_transform_global
                .forward()
                .cross(Vec3::Y)
                .normalize();

        let transform = Transform {
            translation: Vec3 {
                x: message.spawn_location.x,
                y: message.spawn_location.y,
                // maybe makes sense to change this value depending on what direction the player
                // is facing, so particles are not "in the collided object", e.g. not visible
                z: message.spawn_location.z,
            },
            rotation: Quat::from_rotation_arc(
                Vec3::X,
                rotation_towards_player_perpendicular,
            ) + rotation_z,
            ..default()
        };

        // cloning handles in bevy is very cheap. note that handles are just references to the
        // actual asset thats stored somewhere. as always, the handle will get dropped after this
        // call, e.g. at function end
        commands.spawn((
            ParticleEffect::new(bullet_impact_effect_handle.clone()),
            transform,
        ));
    }
}

// i really need to figure out how to dynamically change properties like gradient color of the
// particles
fn setup_player_bullet_impact_enemy_handle(
    mut effects: ResMut<Assets<EffectAsset>>,
    mut player_bullet_impact_enemy: ResMut<
        PlayerBulletHitEnemyImpactEffectHandle,
    >,
) {
    let expression_writer = ExprWriter::new();

    // gradient colors
    let mut gradient = bevy_hanabi::Gradient::new();

    gradient.add_key(0.0, Vec4::new(0.8, 0.0, 0.0, 1.0));
    gradient.add_key(0.25, Vec4::new(0.6, 0.0, 0.0, 0.9));
    gradient.add_key(0.6, Vec4::new(0.3, 0.0, 0.0, 0.7));
    gradient.add_key(1.0, Vec4::new(0.15, 0.05, 0.05, 0.0));

    // where the origin of the particles is (randomly)
    let init_position_modifier = SetPositionCircleModifier {
        center: expression_writer.lit(Vec3::ZERO).expr(),
        axis: expression_writer.lit(Vec3::Z).expr(),
        radius: expression_writer.lit(0.1).expr(),
        dimension: ShapeDimension::Surface,
    };

    // particles "fly" away from center by speed of 3 units/sec
    let init_velocity = SetVelocityCircleModifier {
        center: expression_writer.lit(Vec3::ZERO).expr(),
        axis: expression_writer.lit(Vec3::Z).expr(),
        speed: expression_writer
            .lit(BULLET_IMPACT_PARTICLE_VELOCITY)
            .expr(),
    };

    // how long the particles are visible
    let lifetime_handle = expression_writer
        .lit(BULLET_IMPACT_PARTICLE_LIFETIME)
        .expr();
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

    let module = expression_writer.finish();
    let bullet_impact_effect =
        EffectAsset::new(1000, SpawnerSettings::once(4.0.into()), module)
            .with_name("BulletImpactStraight")
            .init(init_position_modifier)
            .init(init_velocity)
            .init(init_lifetime)
            .init(size_attribute)
            .render(orient_modifier_billboard)
            .render(ColorOverLifetimeModifier {
                gradient,
                ..default()
            });

    let effect_handle = effects.add(bullet_impact_effect);
    player_bullet_impact_enemy.0 = Some(effect_handle);
}
