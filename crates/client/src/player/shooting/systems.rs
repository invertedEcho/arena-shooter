use avian3d::prelude::*;
use bevy::{input::mouse::MouseWheel, prelude::*};
use lightyear::prelude::*;
use shared::{
    components::Health,
    player::{AimType, PlayerState},
    protocol::{OrderedReliableChannel, ShootRequest},
    shooting::MAX_SHOOTING_DISTANCE,
};

use crate::{
    particles::{BulletImpactEffectVariant, SpawnBulletImpactEffectMessage},
    player::{
        Player, PlayerDeathMessage,
        camera::{
            components::{PlayerWeaponModel, WorldCamera},
            weapon_positions::get_position_for_weapon,
        },
        shooting::{
            components::{PlayerShootCooldownTimer, PlayerWeapons, Weapon},
            messages::{
                PlayerBulletHit, PlayerWeaponFiredMessage,
                PlayerWeaponSlotChangeMessage, ReloadPlayerWeaponMessage,
            },
            resources::{ChangeWeaponCooldown, WeaponReloadTimer},
        },
    },
    shared::{
        WeaponSlotType, WeaponState, WeaponStats, WeaponType,
        get_fire_delay_by_weapon_type,
    },
    utils::query_filters::{OurPlayerFilter, PlayerOrEnemyFilter},
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
    (With<WorldCamera>, Without<Player>),
>;

pub fn add_player_weapons_to_new_players(
    added_players: Query<Entity, (Added<Player>, With<Controlled>)>,
    mut commands: Commands,
) {
    for player_entity in added_players {
        commands.entity(player_entity).insert((
            AimType::Normal,
            PlayerWeapons {
                weapons: [
                    Weapon {
                        stats: WeaponStats {
                            weapon_type: WeaponType::AssaultRifle,
                            max_loaded_ammo: 30,
                            weapon_slot_type: WeaponSlotType::Primary,
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
                            weapon_slot_type: WeaponSlotType::Secondary,
                        },
                        state: WeaponState {
                            loaded_ammo: 15,
                            carried_ammo: 50,
                        },
                    },
                ],
            },
        ));
    }
}

pub fn handle_input(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_shot_messsage_writer: MessageWriter<PlayerWeaponFiredMessage>,
    mut reload_player_weapon_message_writer: MessageWriter<
        ReloadPlayerWeaponMessage,
    >,
    player_weapon_shoot_cooldown_timer_query: Query<&PlayerShootCooldownTimer>,
    player_query: Single<
        (&mut PlayerWeapons, &mut PlayerState, &mut AimType),
        OurPlayerFilter,
    >,
) {
    let (mut player_weapons, mut player_state, mut aim_type) =
        player_query.into_inner();

    let already_reloading = player_state.reloading;

    let current_weapon =
        &mut player_weapons.weapons[player_state.active_weapon_slot];
    let current_weapon_stats = &current_weapon.stats;
    let current_weapon_state = &mut current_weapon.state;

    let is_current_weapon_secondary =
        current_weapon_stats.weapon_slot_type == WeaponSlotType::Secondary;

    let weapon_is_full = current_weapon_stats.max_loaded_ammo
        == current_weapon_state.loaded_ammo;

    let shoot_button_pressed = if is_current_weapon_secondary {
        mouse_input.just_pressed(MouseButton::Left)
    } else {
        mouse_input.pressed(MouseButton::Left)
    };

    let reload_button_pressed = keyboard_input.just_pressed(KeyCode::KeyR);

    if shoot_button_pressed {
        player_state.shooting = true;
    } else if mouse_input.just_released(MouseButton::Left) {
        player_state.shooting = false;
    }

    if shoot_button_pressed {
        if player_weapon_shoot_cooldown_timer_query.count() != 0 {
            return;
        }

        // TODO: play a sound which indicates empty magazine
        let active_weapon_empty = current_weapon_state.loaded_ammo == 0;

        if active_weapon_empty {
            return;
        }

        if player_state.reloading {
            return;
        }

        current_weapon_state.loaded_ammo -= 1;

        player_shot_messsage_writer.write(PlayerWeaponFiredMessage);

        let fire_delay =
            get_fire_delay_by_weapon_type(&current_weapon_stats.weapon_type);
        commands.spawn(PlayerShootCooldownTimer(Timer::from_seconds(
            fire_delay,
            TimerMode::Once,
        )));
    }

    if reload_button_pressed && !weapon_is_full && !already_reloading {
        *aim_type = AimType::Normal;
        reload_player_weapon_message_writer.write(ReloadPlayerWeaponMessage);
    }
}

pub fn send_shoot_request_on_weapon_fired(
    mut message_reader: MessageReader<PlayerWeaponFiredMessage>,
    mut shoot_request_sender: Single<
        &mut MessageSender<ShootRequest>,
        With<Client>,
    >,
    world_model_camera_query: WorldModelCameraQuery,
) {
    for _ in message_reader.read() {
        let origin = world_model_camera_query.1.translation();
        let direction = world_model_camera_query.1.forward();

        shoot_request_sender
            .send::<OrderedReliableChannel>(ShootRequest { direction, origin });
    }
}

pub fn check_if_player_bullet_hit(
    mut message_reader: MessageReader<PlayerWeaponFiredMessage>,
    mut message_writer: MessageWriter<PlayerBulletHit>,
    spatial_query: SpatialQuery,
    player_entity: Single<Entity, OurPlayerFilter>,
    world_model_camera_query: WorldModelCameraQuery,
) {
    for _ in message_reader.read() {
        let origin = world_model_camera_query.1.translation();
        let direction = world_model_camera_query.1.forward();

        let Some(first_hit) = spatial_query.cast_ray(
            origin,
            direction,
            MAX_SHOOTING_DISTANCE,
            false,
            &SpatialQueryFilter::default()
                .with_excluded_entities([*player_entity]),
        ) else {
            continue;
        };

        let hit_point = origin + direction * first_hit.distance;
        message_writer.write(PlayerBulletHit {
            hit_point,
            entity_hit: first_hit.entity,
        });
    }
}

pub fn spawn_bullet_impact_particle_on_player_bullet_hit(
    mut message_reader: MessageReader<PlayerBulletHit>,
    mut spawn_bullet_impact_effect_message_writer: MessageWriter<
        SpawnBulletImpactEffectMessage,
    >,
    player_and_enemies: Query<Entity, PlayerOrEnemyFilter>,
) {
    for message in message_reader.read() {
        let player_or_enemy_hit =
            player_and_enemies.get(message.entity_hit).is_ok();

        let bullet_impact_effect_variant = if player_or_enemy_hit {
            BulletImpactEffectVariant::Enemy
        } else {
            BulletImpactEffectVariant::World
        };

        spawn_bullet_impact_effect_message_writer.write(
            SpawnBulletImpactEffectMessage {
                spawn_location: message.hit_point,
                variant: bullet_impact_effect_variant,
            },
        );
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

pub fn handle_reload_player_weapon_message(
    mut commands: Commands,
    player_query: Single<(&mut PlayerWeapons, &mut PlayerState)>,
    mut message_reader: MessageReader<ReloadPlayerWeaponMessage>,
    mut player_weapon_model_transform: Single<
        &mut Transform,
        With<PlayerWeaponModel>,
    >,
) {
    let (mut player_weapons, mut player_state) = player_query.into_inner();
    for _ in message_reader.read() {
        debug!("received reload player weapon message message");

        let active_slot = player_state.active_weapon_slot;

        let player_weapon_state =
            &mut player_weapons.weapons[active_slot].state;

        let reload_timer_duration = if player_weapon_state.loaded_ammo == 0 {
            FULL_RELOAD_TIME
        } else {
            PARTIAL_RELOAD_TIME
        };

        commands.insert_resource(WeaponReloadTimer(Timer::from_seconds(
            reload_timer_duration,
            TimerMode::Once,
        )));

        player_state.reloading = true;

        let weapon_type = &player_weapons.weapons
            [player_state.active_weapon_slot]
            .stats
            .weapon_type;
        let weapon_position =
            get_position_for_weapon(weapon_type, &AimType::Normal);

        player_weapon_model_transform.translation = weapon_position;
    }
}

pub fn handle_player_weapon_reload_timer(
    player_weapons: Single<(&mut PlayerWeapons, &mut PlayerState)>,
    reload_timer: Option<ResMut<WeaponReloadTimer>>,
    time: Res<Time>,
) {
    let Some(mut reload_timer) = reload_timer else {
        return;
    };

    let (mut player_weapons, mut player_state) = player_weapons.into_inner();

    if !player_state.reloading {
        return;
    }

    let timer = &mut reload_timer.0;
    timer.tick(time.delta());

    if timer.just_finished() {
        player_state.reloading = false;

        let active_slot = player_state.active_weapon_slot;

        let weapon_stats = player_weapons.weapons[active_slot].stats.clone();
        let active_weapon_state =
            &mut player_weapons.weapons[active_slot].state;

        let missing_bullets_to_load =
            weapon_stats.max_loaded_ammo - active_weapon_state.loaded_ammo;

        // TODO: i think this can be simplified
        if active_weapon_state.carried_ammo > missing_bullets_to_load {
            active_weapon_state.loaded_ammo += missing_bullets_to_load;
            active_weapon_state.carried_ammo -= missing_bullets_to_load;
        } else {
            active_weapon_state.loaded_ammo = active_weapon_state.carried_ammo;
            active_weapon_state.carried_ammo = 0;
        }
    }
}

pub fn handle_weapon_slot_change(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_scroll_message_reader: MessageReader<MouseWheel>,
    mut player_state: Single<&mut PlayerState, With<Controlled>>,
    mut message_writer: MessageWriter<PlayerWeaponSlotChangeMessage>,
    existing_change_weapon_cooldown: Option<Res<ChangeWeaponCooldown>>,
) {
    // dont allow changing weapon if on cooldown
    if existing_change_weapon_cooldown.is_some() {
        return;
    }

    let current_slot = player_state.active_weapon_slot;
    let mut new_slot = current_slot;

    for mouse_scroll_message in mouse_scroll_message_reader.read() {
        if mouse_scroll_message.y != 0. {
            new_slot = if current_slot == 0 { 1 } else { 0 };
        }
    }

    if keyboard_input.just_pressed(KeyCode::Digit1) {
        new_slot = 0;
    } else if keyboard_input.just_pressed(KeyCode::Digit2) {
        new_slot = 1;
    }

    if current_slot != new_slot {
        player_state.active_weapon_slot = new_slot;
        message_writer.write(PlayerWeaponSlotChangeMessage(new_slot));
        commands.insert_resource(ChangeWeaponCooldown(Timer::from_seconds(
            0.05,
            TimerMode::Once,
        )));
        // cancel any ongoing reload
        commands.remove_resource::<WeaponReloadTimer>();
        player_state.reloading = false;
    }
}

pub fn handle_change_weapon_slot_cooldown(
    mut commands: Commands,
    existing_change_weapon_cooldown: Option<ResMut<ChangeWeaponCooldown>>,
    time: Res<Time>,
) {
    if let Some(mut change_weapon_cooldown) = existing_change_weapon_cooldown {
        change_weapon_cooldown.0.tick(time.delta());
        if change_weapon_cooldown.0.just_finished() {
            commands.remove_resource::<ChangeWeaponCooldown>();
        }
    }
}

pub fn reset_aim_type_on_pause(mut aim_type: Single<&mut AimType>) {
    **aim_type = AimType::Normal;
}

pub fn check_if_player_dead(
    player_health: Single<&Health, (Changed<Health>, With<Controlled>)>,
    mut player_death_message_writer: MessageWriter<PlayerDeathMessage>,
) {
    if player_health.0 <= 0.0 {
        player_death_message_writer.write(PlayerDeathMessage);
    }
}
