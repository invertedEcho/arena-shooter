use avian3d::prelude::*;
use bevy::{input::mouse::MouseWheel, prelude::*};
use shared::player::{AimType, PlayerWeaponModel};

use crate::{
    game_flow::states::InGameState,
    player::{
        Player, PlayerDeathMessage,
        camera::{
            components::WorldCamera, weapon_positions::get_position_for_weapon,
        },
        shooting::{
            components::{
                BloodScreenEffect, PlayerShootCooldownTimer, PlayerWeapons,
                Weapon,
            },
            messages::{
                PlayerWeaponFiredMessage, PlayerWeaponSlotChangeMessage,
                ReloadPlayerWeaponMessage,
            },
            resources::{ChangeWeaponCooldown, WeaponReloadTimer},
        },
    },
    shared::{
        WeaponSlotType, WeaponState, WeaponStats, WeaponType,
        get_fire_delay_by_weapon_type,
    },
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
    added_players: Query<Entity, Added<Player>>,
    mut commands: Commands,
) {
    for player_entity in added_players {
        commands.entity(player_entity).insert(PlayerWeapons {
            shooting: false,
            reloading: false,
            active_slot: 0,
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
    let current_weapon_secondary = player_weapons.weapons
        [player_weapons.active_slot]
        .stats
        .weapon_slot_type
        == WeaponSlotType::Secondary;

    let shoot_button_pressed = if current_weapon_secondary {
        mouse_input.just_pressed(MouseButton::Left)
    } else {
        mouse_input.pressed(MouseButton::Left)
    };

    let reload_button_pressed = keyboard_input.just_pressed(KeyCode::KeyR);

    if mouse_input.just_pressed(MouseButton::Left) {
        player_weapons.shooting = true;
    } else if mouse_input.just_released(MouseButton::Left) {
        player_weapons.shooting = false;
    }

    if shoot_button_pressed {
        if player_weapon_shoot_cooldown_timer_query.iter().len() != 0 {
            return;
        }

        // TODO: play a sound which indicates empty magazine
        let active_weapon_empty = player_weapons.weapons
            [player_weapons.active_slot]
            .state
            .loaded_ammo
            == 0;
        if active_weapon_empty {
            return;
        }

        if player_weapons.reloading {
            return;
        }

        let active_slot = player_weapons.active_slot;
        player_weapons.weapons[active_slot].state.loaded_ammo -= 1;

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
    // enemy_entities: Query<Entity, With<Enemy>>,
    // mut player_bullet_hit_enemy_message_writer: MessageWriter<
    //     PlayerBulletHitEnemyMessage,
    // >,
    // mut spawn_bullet_impact_effect_message_writer: MessageWriter<
    //     SpawnBulletImpactEffectMessage,
    // >,
    world_model_camera_query: WorldModelCameraQuery,
    player_query: Single<(Entity, &PlayerWeapons), With<Player>>,
) {
    let (player_entity, player_weapon) = player_query.into_inner();

    for _ in message_reader.read() {
        let fire_delay = get_fire_delay_by_weapon_type(
            &player_weapon.weapons[player_weapon.active_slot]
                .stats
                .weapon_type,
        );
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

            // let did_hit_enemy =
            //     enemy_entities.iter().any(|e| e == first_hit.entity);

            // if did_hit_enemy {
            //     player_bullet_hit_enemy_message_writer.write(
            //         PlayerBulletHitEnemyMessage {
            //             enemy_hit: entity_hit,
            //             damage: DEFAULT_BULLET_DAMAGE,
            //         },
            //     );
            // }

            let hit_point = origin + direction * first_hit.distance;

            // let variant = if did_hit_enemy {
            //     BulletImpactEffectVariant::Enemy
            // } else {
            //     BulletImpactEffectVariant::World
            // };
            //
            // spawn_bullet_impact_effect_message_writer.write(
            //     SpawnBulletImpactEffectMessage {
            //         spawn_location: hit_point,
            //         variant,
            //     },
            // );
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

// TODO: Move to player/hud module
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
    mut player_weapons: Single<&mut PlayerWeapons>,
    mut message_reader: MessageReader<ReloadPlayerWeaponMessage>,
    mut player_weapon_model_transform: Single<
        &mut Transform,
        With<PlayerWeaponModel>,
    >,
) {
    for _ in message_reader.read() {
        debug!("received reload player weapon message message");
        // dont allow reloading when already reloading
        if player_weapons.reloading {
            debug!("already reloading ignoring reloadmessage");
            return;
        }

        let active_slot = player_weapons.active_slot;
        let player_weapon_stats =
            player_weapons.weapons[active_slot].stats.clone();
        let player_weapon_state =
            &mut player_weapons.weapons[active_slot].state;

        if player_weapon_state.loaded_ammo
            == player_weapon_stats.max_loaded_ammo
        {
            debug!("Player weapon already full");
            return;
        }

        let reload_timer_duration = if player_weapon_state.loaded_ammo == 0 {
            FULL_RELOAD_TIME
        } else {
            PARTIAL_RELOAD_TIME
        };

        commands.insert_resource(WeaponReloadTimer(Timer::from_seconds(
            reload_timer_duration,
            TimerMode::Once,
        )));

        player_weapons.reloading = true;

        let weapon_type = &player_weapons.weapons[player_weapons.active_slot]
            .stats
            .weapon_type;
        let weapon_position =
            get_position_for_weapon(weapon_type, &AimType::Normal);

        player_weapon_model_transform.translation = weapon_position;
    }
}

pub fn tick_player_weapon_reload_timer(
    mut player_weapons: Single<&mut PlayerWeapons>,
    reload_timer: Option<ResMut<WeaponReloadTimer>>,
    time: Res<Time>,
) {
    let Some(mut reload_timer) = reload_timer else {
        return;
    };

    if !player_weapons.reloading {
        return;
    }

    let timer = &mut reload_timer.0;
    timer.tick(time.delta());

    if timer.just_finished() {
        player_weapons.reloading = false;

        let active_slot = player_weapons.active_slot;

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

// TODO: move to game_flow module
pub fn handle_player_death_event(
    mut message_reader: MessageReader<PlayerDeathMessage>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    for _ in message_reader.read() {
        next_in_game_state.set(InGameState::PlayerDead);
    }
}

pub fn handle_weapon_slot_change(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_scroll_message_reader: MessageReader<MouseWheel>,
    mut player_weapons: Single<&mut PlayerWeapons>,
    mut message_writer: MessageWriter<PlayerWeaponSlotChangeMessage>,
    existing_change_weapon_cooldown: Option<Res<ChangeWeaponCooldown>>,
) {
    // dont allow changing weapon if on cooldown
    if existing_change_weapon_cooldown.is_some() {
        return;
    }

    let old_slot = player_weapons.active_slot;
    let mut new_slot = old_slot;

    for mouse_scroll_message in mouse_scroll_message_reader.read() {
        if mouse_scroll_message.y != 0. {
            let current_slot = player_weapons.active_slot;
            new_slot = if current_slot == 0 { 1 } else { 0 };
        }
    }

    if keyboard_input.just_pressed(KeyCode::Digit1) {
        new_slot = 0;
    } else if keyboard_input.just_pressed(KeyCode::Digit2) {
        new_slot = 1;
    }

    if old_slot != new_slot {
        player_weapons.active_slot = new_slot;
        message_writer.write(PlayerWeaponSlotChangeMessage(new_slot));
        commands.insert_resource(ChangeWeaponCooldown(Timer::from_seconds(
            0.05,
            TimerMode::Once,
        )));
        // cancel any ongoing reload
        commands.remove_resource::<WeaponReloadTimer>();
        player_weapons.reloading = false;
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
