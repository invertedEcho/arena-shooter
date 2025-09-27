use std::ops::Neg;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    common::{BULLET_VELOCITY, components::DespawnTimer},
    enemy::{Enemy, shooting::components::EnemyBullet},
    game_flow::{score::GameScore, states::InGameState},
    particles::{BulletImpactEffectVariant, SpawnBulletImpactEffectEvent},
    player::{
        Player,
        camera::components::PlayerCamera,
        shooting::{
            components::{
                BloodScreenEffect, MuzzleFlash, PlayerBullet,
                PlayerShootCooldownTimer, PlayerWeapon,
            },
            events::PlayerWeaponFiredEvent,
        },
    },
    utils::random::get_random_number_from_range_i32,
};

pub fn player_shooting(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    player_camera_global_transform: Single<
        &GlobalTransform,
        (With<PlayerCamera>, Without<Player>),
    >,
    player_weapon_shoot_cooldown_timer_query: Query<&PlayerShootCooldownTimer>,
    mut player_weapon: Single<&mut PlayerWeapon>,
    mut player_shot_event_writer: EventWriter<PlayerWeaponFiredEvent>,
) {
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    if player_weapon_shoot_cooldown_timer_query.iter().len() != 0 {
        return;
    }

    // TODO: play a sound which indicates empty magazine
    if player_weapon.loaded_ammo == 0 {
        return;
    }
    player_weapon.loaded_ammo -= 1;

    commands.spawn(PlayerShootCooldownTimer(Timer::from_seconds(
        0.1,
        TimerMode::Once,
    )));

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
        PlayerBullet { damage: 15.0 },
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
        LinearVelocity(world_bullet_velocity),
        RigidBody::Kinematic,
        DespawnTimer(Timer::from_seconds(3.0, TimerMode::Once)),
        CollisionEventsEnabled,
    ));

    player_shot_event_writer.write(PlayerWeaponFiredEvent);
}

pub fn tick_player_weapon_shoot_cooldown_timer(
    mut commands: Commands,
    query: Query<(Entity, &mut PlayerShootCooldownTimer)>,
    time: Res<Time>,
) {
    for (entity, mut timer) in query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn detect_enemy_bullet_collision_with_player(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut collision_event_reader: EventReader<CollisionStarted>,
    enemy_bullet_query: Query<Entity, With<EnemyBullet>>,
    player_query: Single<(Entity, &mut Player)>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
    mut game_score: ResMut<GameScore>,
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

        let Some(bullet_entity) = enemy_bullet_query
            .iter()
            .find(|entity| entity == first_entity || entity == second_entity)
        else {
            continue;
        };
        commands.entity(bullet_entity).despawn();

        commands.spawn((
            ImageNode {
                image: asset_server.load("Bloody Screen Effects/Effect_5.png"),
                color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                ..default()
            },
            BloodScreenEffect::default(),
        ));

        player.health -= 10.0;
        if player.health <= 0.0 {
            next_in_game_state.set(InGameState::PlayerDead);
            game_score.enemy += 1;
        }

        // TODO: bullet damage indicator directional to enemy:
        // Add once bevy 0.17 is released and all dependencies this project uses have migrated to
        // 0.17
        // let Ok(transform_of_enemy) =
        //     enemy_transforms.get(enemy_bullet.origin_enemy)
        // else {
        //     continue;
        // };
        // let hit_direction = (transform_of_enemy.translation
        //     - player_transform.translation)
        //     .normalize();
        // let local_direction =
        //     player_transform.rotation.conjugate() * hit_direction;
        // let flat_direction = Vec2::new(local_direction.x, local_direction.z);
        // let angle_radians = flat_direction.x.atan2(flat_direction.y);
        // let angle_degrees = angle_radians.to_degrees();
        // commands
        //     .spawn(Node {
        //         width: Val::Percent(100.0),
        //         height: Val::Percent(100.0),
        //         justify_content: JustifyContent::Center,
        //         align_items: AlignItems::Center,
        //         ..default()
        //     })
        //     .with_child((
        //         ImageNode {
        //             image: asset_server.load("hud/damage_indicator.png"),
        //             ..default()
        //         },
        //         UiTransform::new(),
        //         BloodScreenEffect::default(),
        //     ));
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

pub fn spawn_muzzle_flash(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_shot_event_reader: EventReader<PlayerWeaponFiredEvent>,
    player_camera_entity: Single<Entity, With<PlayerCamera>>,
) {
    for _ in player_shot_event_reader.read() {
        let random_rotation_angle = get_random_number_from_range_i32(0, 5);
        commands.entity(*player_camera_entity).with_child((
            Transform {
                // TODO: this is really not good. we need a better way to get the correct position
                // to spawn the muzzle flash to relative to player view. this must also change
                // depending on the cameras FOV
                translation: Vec3 {
                    x: 0.3,
                    y: -0.1,
                    z: -0.5,
                },
                rotation: Quat::from_axis_angle(
                    Vec3::Z,
                    random_rotation_angle as f32,
                ),
                ..default()
            },
            MuzzleFlash,
            Mesh3d(meshes.add(Plane3d {
                half_size: Vec2::splat(0.1),
                normal: Dir3::Z,
                ..default()
            })),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(
                    asset_server.load("muzzle_flash_cropped.png"),
                ),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            DespawnTimer(Timer::from_seconds(0.05, TimerMode::Once)),
        ));
    }
}

/// cast a ray in direction player is shooting, to check if there is a wall or ground, and get
/// accurate location to know where to spawn the bullet impact effect
/// just checking for collision events doesnt work, as we would only get the center transform of the
/// collided entity, which may be very inaccurate, as the object may be large
// need to check if world or enemy
pub fn accurate_check_bullet_collision_for_impact_particle(
    spatial_query: SpatialQuery,
    player_entity: Single<Entity, With<Player>>,
    mut bullet_effect_spawn_event_writer: EventWriter<
        SpawnBulletImpactEffectEvent,
    >,
    enemy_entities: Query<Entity, With<Enemy>>,
    player_camera_query: Single<
        (Entity, &GlobalTransform),
        (With<PlayerCamera>, Without<Player>),
    >,
    mut player_shot_event_reader: EventReader<PlayerWeaponFiredEvent>,
    // maybe only include player bullets. would be cool to be able to shoot enemy bullets and have
    // a special effect or something
    bullet_entities: Query<Entity, Or<(With<PlayerBullet>, With<EnemyBullet>)>>,
) {
    for _ in player_shot_event_reader.read() {
        let (player_camera_entity, player_camera_global_transform) =
            *player_camera_query;

        // ray-cast settings
        let origin = player_camera_global_transform.translation();
        let direction = player_camera_global_transform.forward();
        let max_distance = 100.0;
        let solid = true;

        let bullet_entities: Vec<Entity> = bullet_entities.iter().collect();
        let filter = SpatialQueryFilter::default().with_excluded_entities(
            [vec![*player_entity, player_camera_entity], bullet_entities]
                .concat(),
        );

        if let Some(first_hit) = spatial_query.cast_ray(
            origin,
            direction,
            max_distance,
            solid,
            &filter,
        ) {
            let did_hit_enemy =
                enemy_entities.iter().any(|e| e == first_hit.entity);

            let hit_point = origin + direction * first_hit.distance;

            let variant = if did_hit_enemy {
                BulletImpactEffectVariant::Enemy
            } else {
                BulletImpactEffectVariant::World
            };

            bullet_effect_spawn_event_writer.write(
                SpawnBulletImpactEffectEvent {
                    spawn_location: hit_point,
                    variant,
                },
            );
        }
    }
}
