use bevy::prelude::*;

use crate::{
    game_flow::{game_mode::GameMode, states::InGameState},
    player::hud::systems::{
        spawn_bullet_hit_crosshair, spawn_player_hud, spawn_score_hud,
        spawn_wave_info_hud, update_player_ammo_text,
        update_player_health_text, update_score_hud, update_wave_info_hud,
    },
};

mod components;
mod systems;

// TODO: assert that only one player/score/wave_info hud can exist

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
                .run_if(in_state(InGameState::Playing)),
        )
        .add_systems(OnEnter(InGameState::Playing), spawn_player_hud)
        .add_systems(
            OnEnter(GameMode::Waves),
            (spawn_wave_info_hud, spawn_score_hud),
        )
        .add_systems(
            Update,
            (update_wave_info_hud).run_if(in_state(GameMode::Waves)),
        );
    }
}
