use bevy::prelude::*;

use crate::{
    game_flow::{game_mode::GameModeState, states::InGameState},
    player::hud::systems::{
        hide_player_crosshair, hide_player_hud, show_player_crosshair,
        show_player_hud, spawn_bullet_hit_crosshair, spawn_player_crosshair,
        spawn_player_hud, spawn_score_hud, spawn_wave_info_hud,
        update_player_ammo_text, update_player_crosshair_visibility,
        update_player_health_text, update_score_hud, update_selected_weapon,
        update_wave_info_hud,
    },
};

mod components;
mod systems;

// TODO: assert that only one player/score/wave_info hud can exist

const MAIN_CROSSHAIR_PATH: &str = "hud/crosshairs/PNG/White/crosshair086.png";
const CROSSHAIR_BULLET_HIT_PATH: &str =
    "hud/crosshairs/PNG/White/crosshair002.png";

pub struct PlayerHudPlugin;

impl Plugin for PlayerHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_player_health_text,
                update_player_ammo_text,
                spawn_bullet_hit_crosshair,
                update_score_hud,
                update_player_crosshair_visibility,
                spawn_player_crosshair,
                update_selected_weapon,
            )
                .run_if(in_state(InGameState::Playing)),
        )
        .add_systems(Update, spawn_player_hud)
        .add_systems(
            OnEnter(GameModeState::Waves),
            (spawn_wave_info_hud, spawn_score_hud),
        )
        .add_systems(
            Update,
            (update_wave_info_hud).run_if(in_state(GameModeState::Waves)),
        )
        .add_systems(
            OnEnter(InGameState::Playing),
            (show_player_hud, show_player_crosshair),
        )
        .add_systems(
            OnExit(InGameState::Playing),
            (hide_player_hud, hide_player_crosshair),
        );
    }
}
