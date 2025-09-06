use avian3d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};

use crate::{
    common::components::DespawnTimer,
    player::{
        Player,
        camera::components::PlayerCamera,
        shooting::components::{
            Bullet, MuzzleFlash, PlayerWeaponShootCooldownTimer,
        },
    },
};

pub fn basic_shooting(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    player_camera_global_transform: Single<
        &GlobalTransform,
        (With<PlayerCamera>, Without<Player>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_weapon_shoot_cooldown_timer_query: Query<
        &PlayerWeaponShootCooldownTimer,
    >,
) {
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    // if on cooldown, dont allow shooting
    if player_weapon_shoot_cooldown_timer_query.iter().len() != 0 {
        return;
    }

    // if no timer, means we are allowed to shoot, and insert the cooldown timer
    commands.spawn(PlayerWeaponShootCooldownTimer(Timer::from_seconds(
        0.1,
        TimerMode::Once,
    )));

    commands.spawn((
        ImageNode {
            image: asset_server.load("muzzle_flash.png"),

            ..default()
        },
        MuzzleFlash,
        DespawnTimer(Timer::from_seconds(0.02, TimerMode::Once)),
    ));

    let audio = asset_server
        .load("weapons/Snake's Authentic Gun Sounds/Full Sound/7.62x39/MP3/762x39 Single MP3.mp3");

    commands.spawn((AudioPlayer::new(audio), PlaybackSettings::ONCE));

    let local_bullet_velocity = Vec3 {
        z: -100.0,
        x: 0.0,
        y: 0.0,
    };
    let world_bullet_velocity =
        player_camera_global_transform.rotation() * local_bullet_velocity;

    let player_camera_global_transform_translation =
        player_camera_global_transform.translation();

    commands.spawn((
        Transform {
            translation: Vec3 {
                x: player_camera_global_transform_translation.x,
                y: player_camera_global_transform_translation.y,
                z: player_camera_global_transform_translation.z,
            },
            ..default()
        },
        Collider::cuboid(0.1, 0.1, 0.1),
        Sensor,
        Mesh3d(meshes.add(Cuboid {
            half_size: Vec3::splat(0.05),
        })),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: RED.into(),
            ..Default::default()
        })),
        LinearVelocity(world_bullet_velocity),
        RigidBody::Kinematic,
        DespawnTimer(Timer::from_seconds(3.0, TimerMode::Once)),
        Bullet,
    ));
}

pub fn billboad_muzzle_flash(
    billboads: Query<&mut Transform, With<MuzzleFlash>>,
    camera_transform: Single<&Transform, (With<Camera>, Without<MuzzleFlash>)>,
) {
    for mut transform in billboads {
        let direction = camera_transform.translation - transform.translation;
        transform.look_at(camera_transform.translation, Vec3::Y);

        transform.rotation =
            Quat::from_rotation_y(direction.x.atan2(direction.z));
    }
}

pub fn tick_player_weapon_timer(
    mut commands: Commands,
    query: Query<(Entity, &mut PlayerWeaponShootCooldownTimer)>,
    time: Res<Time>,
) {
    for (entity, mut timer) in query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn detect_bullet_collision_with_player(
    mut collision_event_reader: EventReader<CollisionStarted>,
    bullet_query: Query<Entity, With<Bullet>>,
    player_entity: Single<Entity, With<Player>>,
) {
    for CollisionStarted(first_entity, second_entity) in
        collision_event_reader.read()
    {
        let collided_entities_is_player =
            *player_entity == *first_entity || *player_entity == *second_entity;

        if !collided_entities_is_player {
            continue;
        }

        let collided_entities_is_bullet = bullet_query
            .iter()
            .any(|bullet| bullet == *first_entity || bullet == *second_entity);
        if !collided_entities_is_bullet {
            continue;
        }

        info!("Player was hit by bullet!");
    }
}
