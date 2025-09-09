use avian3d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};

use crate::{
    common::components::DespawnTimer,
    enemy::EnemyBullet,
    player::{
        Player,
        camera::components::PlayerCamera,
        shooting::components::{
            BloodScreenEffect, MuzzleFlash, PlayerBullet,
            PlayerWeaponShootCooldownTimer,
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
        PlayerBullet,
    ));
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

// TODO: Also despawn bullet if player hit
pub fn detect_bullet_collision_with_player(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut collision_event_reader: EventReader<CollisionStarted>,
    enemy_bullet_query: Query<Entity, With<EnemyBullet>>,
    player_query: Single<(Entity, &mut Player)>,
) {
    let (player_entity, mut player) = player_query.into_inner();

    for CollisionStarted(first_entity, second_entity) in
        collision_event_reader.read()
    {
        let collided_entities_is_player =
            player_entity == *first_entity || player_entity == *second_entity;

        if !collided_entities_is_player {
            continue;
        }

        let collided_entities_is_bullet = enemy_bullet_query
            .iter()
            .any(|bullet| bullet == *first_entity || bullet == *second_entity);
        if !collided_entities_is_bullet {
            continue;
        }

        info!("Player was hit by enemy bullet!");
        // how to do fade out? i guess timer repeating every 0.1 second further decrease alpha,
        // from 1.0 to 0.0, so timer needs to run ten times. also, when player hit again, we should
        // despawn current blood screen effect and spawn new and start from 1.0 again
        commands.spawn((
            ImageNode {
                image: asset_server.load("Bloody Screen Effects/Effect_5.png"),
                color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                ..default()
            },
            BloodScreenEffect::default(),
        ));

        if player.health == 0 {
            warn!("Player already dead, ignoring bullet collision event");
            continue;
        }
        player.health -= 10;
    }
}

pub fn handle_blood_screen_effect(
    mut blood_screen_effect_query: Query<(
        Entity,
        &mut BloodScreenEffect,
        &mut ImageNode,
    )>,
    mut commands: Commands,
    time: Res<Time>,
) {
    // TODO: hmm i mean theoretically only one instance of blood screen effect should exist, maybe
    // convert to `Resource`?
    for (entity, mut blood_screen_effect, mut image_node) in
        blood_screen_effect_query.iter_mut()
    {
        let timer = &mut blood_screen_effect.timer;
        timer.tick(time.delta());
        if timer.just_finished() {
            let new_current_timer_iteration =
                blood_screen_effect.currrent_timer_iteration + 1;
            if new_current_timer_iteration as f32
                > blood_screen_effect.total_timer_iteration_count
            {
                commands.entity(entity).despawn();
                continue;
            }
            let current_color = &image_node.color;
            image_node.color =
                Color::srgba(1.0, 1.0, 1.0, current_color.alpha() - 0.1);
        }
    }
}
