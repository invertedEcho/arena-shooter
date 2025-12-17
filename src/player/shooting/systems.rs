use std::thread::current;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    character_controller::components::MovementState,
    enemy::Enemy,
    game_flow::states::InGameState,
    particles::{BulletImpactEffectVariant, SpawnBulletImpactEffectMessage},
    player::{
        Player, PlayerDeathMessage,
        camera::{
            components::{
                PlayerWeaponModel, ViewModelCamera, WorldModelCamera,
            },
            weapon_positions::{AimType, get_position_for_weapon},
        },
        shooting::{
            components::{
                BloodScreenEffect, MuzzleFlash, PlayerShootCooldownTimer,
                PlayerWeapons, Weapon,
            },
            messages::{
                PlayerBulletHitEnemyMessage, PlayerWeaponFiredMessage,
                ReloadPlayerWeaponMessage,
            },
            resources::PlayerWeaponReloadTimer,
        },
    },
    shared::{
        DEFAULT_BULLET_DAMAGE, WeaponSlot, WeaponState, WeaponStats,
        WeaponType, components::DespawnTimer, get_fire_delay_by_weapon_type,
    },
    utils::random::get_random_number_from_range,
};

/// How long it takes to reload for a partial reload (and playing the corresponding animation), e.g. some bullets are left in
/// the player weapon
const PARTIAL_RELOAD_TIME: f32 = 2.81;
/// How long it takes to reload for a full reload (and playing the corresponding animation), e.g. player's weapon is empty
const FULL_RELOAD_TIME: f32 = 3.65;

type WorldModelCameraQuery<'w, 's> = Single<
    'w,
    's,
    (Entity, &'static GlobalTransform),
    (With<WorldModelCamera>, Without<Player>),
>;

pub fn add_player_weapons_to_new_players(
    added_players: Query<Entity, Added<Player>>,
    mut commands: Commands,
) {
    for player_entity in added_players {
        commands.entity(player_entity).insert(PlayerWeapons {
            active: WeaponSlot::Primary,
            weapons: [
                Weapon {
                    stats: WeaponStats {
                        weapon_type: WeaponType::AssaultRifle,
                        max_loaded_ammo: 30,
                    },
                    state: WeaponState {
                        loaded_ammo: 30,
                        carried_ammo: 120,
                    },
                },
                Weapon {
                    stats: WeaponStats {
                        weapon_type: WeaponType::Pistol,
                        max_loaded_ammo: 15,
                    },
                    state: WeaponState {
                        loaded_ammo: 15,
                        carried_ammo: 50,
                    },
                },
            ],
        });
    }
}

pub fn handle_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_shot_messsage_writer: MessageWriter<PlayerWeaponFiredMessage>,
    mut reload_player_weapon_message_writer: MessageWriter<
        ReloadPlayerWeaponMessage,
    >,
    player_weapon_shoot_cooldown_timer_query: Query<&PlayerShootCooldownTimer>,
    mut player_weapons: Single<&mut PlayerWeapons>,
) {
    let current_weapon_secondary =
        player_weapons.active == WeaponSlot::Secondary;

    let shoot_button_pressed = if current_weapon_secondary {
        mouse_input.just_pressed(MouseButton::Left)
    } else {
        mouse_input.pressed(MouseButton::Left)
    };

    let reload_button_pressed = keyboard_input.just_pressed(KeyCode::KeyR);

    if mouse_input.just_pressed(MouseButton::Left) {
        player_weapons.is_shooting = true;
    } else if mouse_input.just_released(MouseButton::Left) {
        player_weapons.is_shooting = false;
    }

    if shoot_button_pressed {
        if player_weapon_shoot_cooldown_timer_query.iter().len() != 0 {
            return;
        }

        // TODO: play a sound which indicates empty magazine
        if player_weapons.weapons[].state.loaded_ammo == 0 {
            return;
        }

        if player_weapons.is_reloading {
            return;
        }

        current_player_weapon.state.loaded_ammo -= 1;

        player_shot_messsage_writer.write(PlayerWeaponFiredMessage);
    }

    if reload_button_pressed {
        reload_player_weapon_message_writer.write(ReloadPlayerWeaponMessage);
    }
}

pub fn handle_player_weapon_fired_message(
    mut commands: Commands,
    spatial_query: SpatialQuery,
    mut message_reader: MessageReader<PlayerWeaponFiredMessage>,
    enemy_entities: Query<Entity, With<Enemy>>,
    mut player_bullet_hit_enemy_message_writer: MessageWriter<
        PlayerBulletHitEnemyMessage,
    >,
    mut spawn_bullet_impact_effect_message_writer: MessageWriter<
        SpawnBulletImpactEffectMessage,
    >,
    world_model_camera_query: WorldModelCameraQuery,
    player_query: Single<(Entity, &PlayerWeapon), With<Player>>,
) {
    let (player_entity, player_weapon) = player_query.into_inner();

    for _ in message_reader.read() {
        let fire_delay =
            get_fire_delay_by_weapon_type(&player_weapon.weapon_type);
        commands.spawn(PlayerShootCooldownTimer(Timer::from_seconds(
            fire_delay,
            TimerMode::Once,
        )));

        let origin = world_model_camera_query.1.translation();
        let direction = world_model_camera_query.1.forward();

        if let Some(first_hit) = spatial_query.cast_ray(
            origin,
            direction,
            500.0,
            false,
            &SpatialQueryFilter::default()
                .with_excluded_entities([player_entity]),
        ) {
            let entity_hit = first_hit.entity;

            let did_hit_enemy =
                enemy_entities.iter().any(|e| e == first_hit.entity);

            if did_hit_enemy {
                player_bullet_hit_enemy_message_writer.write(
                    PlayerBulletHitEnemyMessage {
                        enemy_hit: entity_hit,
                        damage: DEFAULT_BULLET_DAMAGE,
                    },
                );
            }

            let hit_point = origin + direction * first_hit.distance;

            let variant = if did_hit_enemy {
                BulletImpactEffectVariant::Enemy
            } else {
                BulletImpactEffectVariant::World
            };

            spawn_bullet_impact_effect_message_writer.write(
                SpawnBulletImpactEffectMessage {
                    spawn_location: hit_point,
                    variant,
                },
            );
        }
    }
}

// TODO: thereotically we could use our DespawnTimer to despawn some "blocking" entity
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

// TODO: this thing is too much (visually) -> only show it when player dead
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

pub fn handle_reload_player_weapon_message(
    mut commands: Commands,
    mut player_weapon: Single<&mut PlayerWeapon>,
    mut message_reader: MessageReader<ReloadPlayerWeaponMessage>,
    mut player_weapon_model_transform: Single<
        &mut Transform,
        With<PlayerWeaponModel>,
    >,
) {
    for _ in message_reader.read() {
        // dont allow reloading when already reloading
        if player_weapon.reloading {
            return;
        }

        if player_weapon.loaded_ammo == player_weapon.max_loaded_ammo {
            return;
        }

        let reload_timer_duration = if player_weapon.loaded_ammo == 0 {
            FULL_RELOAD_TIME
        } else {
            PARTIAL_RELOAD_TIME
        };

        commands.insert_resource(PlayerWeaponReloadTimer(Timer::from_seconds(
            reload_timer_duration,
            TimerMode::Once,
        )));

        player_weapon.reloading = true;

        let weapon_position = get_position_for_weapon(
            &player_weapon.weapon_type,
            AimType::Normal,
        );
        player_weapon_model_transform.translation = weapon_position;
    }
}

pub fn tick_player_weapon_reload_timer(
    mut player_weapon: Single<&mut PlayerWeapon>,
    reload_timer: Option<ResMut<PlayerWeaponReloadTimer>>,
    time: Res<Time>,
) {
    let Some(mut reload_timer) = reload_timer else {
        return;
    };

    if !player_weapon.reloading {
        return;
    }
    reload_timer.0.tick(time.delta());
    if reload_timer.0.just_finished() {
        player_weapon.reloading = false;

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
}

pub fn play_shooting_sound_on_player_weapon_fired(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut message_reader: MessageReader<PlayerWeaponFiredMessage>,
) {
    for _ in message_reader.read() {
        let shoot_sound = asset_server.load(
            "sfx/Snake's Authentic Gun Sounds/Full Sound/7.62x39/MP3/762x39 \
             Single MP3.mp3",
        );

        commands.spawn((
            AudioPlayer::new(shoot_sound),
            PlaybackSettings::ONCE,
            Name::new("shoot sound player"),
            DespawnTimer(Timer::from_seconds(2.0, TimerMode::Once)),
        ));
    }
}

pub fn spawn_muzzle_flash(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_shot_message_reader: MessageReader<PlayerWeaponFiredMessage>,
    player_camera_entity: Single<Entity, With<ViewModelCamera>>,
) {
    for _ in player_shot_message_reader.read() {
        let random_rotation_angle = get_random_number_from_range(0..5);

        commands.entity(*player_camera_entity).with_child((
            Transform {
                // TODO: this must change depending on the cameras FOV
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
            })),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(
                    // TODO: dont use cropped version to avoid the bleeding
                    // thing
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

pub fn handle_player_death_event(
    mut message_reader: MessageReader<PlayerDeathMessage>,
    mut player_movement_state: Single<&mut MovementState, With<Player>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    for _ in message_reader.read() {
        **player_movement_state = MovementState::Idle;
        next_in_game_state.set(InGameState::PlayerDead);
    }
}
