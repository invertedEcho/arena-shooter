use bevy::prelude::*;

use crate::{
    game_flow::states::InGameState,
    player::shooting::{
        components::PlayerWeapon,
        messages::{
            PlayerBulletHitEnemyMessage, PlayerWeaponFiredMessage,
            ReloadPlayerWeaponMessage,
        },
        systems::{
            handle_blood_screen_effect, handle_input,
            handle_player_death_event, handle_player_weapon_fired_message,
            handle_reload_player_weapon_message,
            play_shooting_sound_on_player_weapon_fired, setup_player_weapon,
            spawn_muzzle_flash, tick_player_weapon_reload_timer,
            tick_player_weapon_shoot_cooldown_timer, weapon_sway,
        },
    },
};

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
            .register_type::<PlayerWeapon>()
            .add_systems(
                Update,
                (
                    handle_input,
                    tick_player_weapon_shoot_cooldown_timer,
                    handle_blood_screen_effect,
                    handle_reload_player_weapon_message,
                    spawn_muzzle_flash,
                    handle_player_weapon_fired_message,
                    setup_player_weapon,
                    tick_player_weapon_reload_timer,
                    handle_player_death_event,
                    play_shooting_sound_on_player_weapon_fired,
                    weapon_sway,
                )
                    .run_if(in_state(InGameState::Playing)),
            );
    }
}
