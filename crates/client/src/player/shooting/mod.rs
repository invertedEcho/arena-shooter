use bevy::prelude::*;

use crate::{
    game_flow::states::InGameState,
    player::shooting::{
        messages::{
            PlayerBulletHit, PlayerWeaponFiredMessage,
            PlayerWeaponSlotChangeMessage, ReloadPlayerWeaponMessage,
        },
        systems::{
            check_if_player_bullet_hit, check_if_player_dead,
            handle_change_weapon_slot_cooldown, handle_input,
            handle_player_scope_aim, handle_player_weapon_reload_timer,
            handle_reload_player_weapon_message, handle_weapon_slot_change,
            reset_aim_type_on_pause, send_shoot_request_on_weapon_fired,
            setup_new_players, spawn_bullet_hole_decal,
            spawn_bullet_impact_particle_on_player_bullet_hit,
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
            .add_message::<PlayerBulletHit>()
            .add_message::<ReloadPlayerWeaponMessage>()
            .add_message::<PlayerWeaponSlotChangeMessage>()
            .add_systems(
                Update,
                (
                    handle_input,
                    tick_player_weapon_shoot_cooldown_timer,
                    handle_reload_player_weapon_message,
                    spawn_bullet_impact_particle_on_player_bullet_hit,
                    setup_new_players,
                    handle_player_weapon_reload_timer,
                    handle_weapon_slot_change,
                    handle_change_weapon_slot_cooldown,
                    send_shoot_request_on_weapon_fired,
                    check_if_player_bullet_hit,
                    spawn_bullet_hole_decal,
                    handle_player_scope_aim,
                )
                    .run_if(in_state(InGameState::Playing)),
            )
            .add_systems(Update, check_if_player_dead)
            .add_systems(OnEnter(InGameState::Paused), reset_aim_type_on_pause);
    }
}
