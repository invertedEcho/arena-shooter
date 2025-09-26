use bevy::prelude::*;

use crate::{
    game_flow::{
        game_mode::GameMode,
        states::{AppState, InGameState},
    },
    player::hud::systems::{
        despawn_player_hud, spawn_bullet_hit_crosshair, spawn_player_hud,
        spawn_score_hud, spawn_wave_info_hud, update_player_ammo_text,
        update_player_health_text, update_score_hud, update_wave_info_hud,
    },
};

mod components;
mod systems;

const WHITE_CROSSHAIR_PATH: &str =
    "kenney_crosshair-pack/PNG/White/crosshair086.png";

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
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            OnEnter(AppState::InGame),
            (spawn_player_hud, spawn_score_hud),
        )
        .add_systems(
            OnExit(InGameState::PlayerDead),
            (spawn_player_hud, spawn_score_hud),
        )
        .add_systems(
            OnExit(InGameState::Paused),
            (spawn_player_hud, spawn_score_hud),
        )
        .add_systems(OnExit(InGameState::Playing), despawn_player_hud)
        .add_systems(OnEnter(GameMode::Waves), spawn_wave_info_hud)
        .add_systems(
            Update,
            (update_wave_info_hud).run_if(in_state(GameMode::Waves)),
        );
    }
}
