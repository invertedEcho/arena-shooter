use std::ops::Neg;

use avian3d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};

use crate::{
    common::{BULLET_VELOCITY, components::DespawnTimer},
    enemy::EnemyBullet,
    player::{
        Player,
        camera::components::PlayerCamera,
        shooting::components::{
            BloodScreenEffect, MuzzleFlash, PlayerBullet, PlayerWeapon,
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
    player_entity: Single<Entity, With<Player>>,
    mut player_weapon: Single<&mut PlayerWeapon>,
) {
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    if player_weapon_shoot_cooldown_timer_query.iter().len() != 0 {
        return;
    }

    if player_weapon.loaded_ammo == 0 {
        return;
    }
    player_weapon.loaded_ammo -= 1;

    commands.spawn(PlayerWeaponShootCooldownTimer(Timer::from_seconds(
        0.1,
        TimerMode::Once,
    )));

    // let random_rotation_angle = get_random_number_from_range_i32(1, 5);

    commands.entity(*player_entity).with_child((
        Sprite {
            image: asset_server.load("muzzle_flash.png"),
            ..default()
        },
        MuzzleFlash,
        Transform {
            // scale: Vec3::splat(0.15),
            // rotation: Quat::from_axis_angle(
            //     Vec3::new(0.0, 0.0, 1.0),
            //     random_rotation_angle as f32,
            // ),
            translation: Vec3::new(0.0, 0.2, -1.0),
            ..default()
        },
        // DespawnTimer(Timer::from_seconds(0.05, TimerMode::Once)),
    ));

    let audio = asset_server
        .load("weapons/Snake's Authentic Gun Sounds/Full Sound/7.62x39/MP3/762x39 Single MP3.mp3");

    commands.spawn((AudioPlayer::new(audio), PlaybackSettings::ONCE));

    let local_bullet_velocity = Vec3 {
        z: BULLET_VELOCITY.neg(),
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
        PlayerBullet { damage: 20.0 },
        CollisionEventsEnabled,
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

        commands.spawn((
            ImageNode {
                image: asset_server.load("Bloody Screen Effects/Effect_5.png"),
                color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                ..default()
            },
            BloodScreenEffect::default(),
        ));

        player.health -= 10.0;
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

pub fn reload_player_weapon(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_weapon: Single<&mut PlayerWeapon>,
) {
    if !keyboard_input.just_pressed(KeyCode::KeyR) {
        return;
    }

    if player_weapon.loaded_ammo == player_weapon.max_loaded_ammo {
        return;
    }

    let missing_bullets_to_load =
        player_weapon.max_loaded_ammo - player_weapon.loaded_ammo;

    if player_weapon.carried_ammo > missing_bullets_to_load {
        player_weapon.loaded_ammo += missing_bullets_to_load;
        player_weapon.carried_ammo -= missing_bullets_to_load;
    } else {
        player_weapon.loaded_ammo = player_weapon.carried_ammo;
        player_weapon.carried_ammo = 0;
    }
}
