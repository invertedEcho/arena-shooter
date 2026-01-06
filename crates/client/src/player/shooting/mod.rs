use bevy::prelude::*;

use crate::{
    game_flow::states::InGameState,
    player::shooting::{
        messages::{
            PlayerBulletHitEnemyMessage, PlayerWeaponFiredMessage,
            PlayerWeaponSlotChangeMessage, ReloadPlayerWeaponMessage,
        },
        systems::{
            add_player_weapons_to_new_players, check_if_player_dead,
            handle_blood_screen_effect, handle_change_weapon_slot_cooldown,
            handle_input, handle_player_death_event,
            handle_player_weapon_fired_message,
            handle_reload_player_weapon_message, handle_weapon_slot_change,
            reset_aim_type_on_pause, tick_player_weapon_reload_timer,
            tick_player_weapon_shoot_cooldown_timer,
        },
    },
};

pub mod asset_paths;
pub mod components;
pub mod messages;
mod resources;
mod systems;

pub struct PlayerShootingPlugin;

impl Plugin for PlayerShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayerWeaponFiredMessage>()
            .add_message::<PlayerBulletHitEnemyMessage>()
            .add_message::<ReloadPlayerWeaponMessage>()
            .add_message::<PlayerWeaponSlotChangeMessage>()
            .add_systems(
                Update,
                (
                    handle_input,
                    tick_player_weapon_shoot_cooldown_timer,
                    handle_blood_screen_effect,
                    handle_reload_player_weapon_message,
                    handle_player_weapon_fired_message,
                    add_player_weapons_to_new_players,
                    tick_player_weapon_reload_timer,
                    handle_player_death_event,
                    handle_weapon_slot_change,
                    handle_change_weapon_slot_cooldown,
                    check_if_player_dead,
                )
                    .run_if(in_state(InGameState::Playing)),
            )
            .add_systems(OnEnter(InGameState::Paused), reset_aim_type_on_pause);
    }
}
