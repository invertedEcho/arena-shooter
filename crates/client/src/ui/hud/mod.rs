use bevy::prelude::*;
use game_core::GameStateWave;
use shared::NextWaveTimer;

use crate::{
    game_flow::states::{AppState, InGameState},
    ui::{
        UiState,
        hud::systems::{
            fade_out_damage_indicator, hide_current_wave_finished_text,
            reflect_crosshair_visibility_from_ui_state,
            show_wave_finished_text, spawn_bullet_hit_crosshair,
            spawn_damage_indicator, spawn_info_text_current_wave_finished,
            spawn_player_crosshair, spawn_player_hud, spawn_wave_hud,
            update_crosshair_visibility_on_aim_type_change,
            update_current_cash_amount, update_next_wave_timer_text,
            update_player_ammo_text, update_player_health_text,
            update_selected_weapon, update_wave_hud,
        },
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
        app.add_systems(Startup, spawn_info_text_current_wave_finished);
        app.add_systems(
            Update,
            (
                update_player_health_text,
                update_player_ammo_text,
                spawn_bullet_hit_crosshair,
                update_crosshair_visibility_on_aim_type_change,
                spawn_player_crosshair,
                update_selected_weapon,
                spawn_damage_indicator,
                fade_out_damage_indicator,
                update_current_cash_amount,
                update_next_wave_timer_text,
                show_wave_finished_text,
            )
                .run_if(in_state(InGameState::Playing)),
        );
        app.add_systems(Update, spawn_player_hud);
        app.add_systems(OnEnter(AppState::InGame), spawn_wave_hud);
        app.add_systems(
            Update,
            (update_wave_hud)
                .run_if(resource_exists_and_changed::<GameStateWave>),
        );
        app.add_systems(
            Update,
            reflect_crosshair_visibility_from_ui_state.run_if(
                resource_changed::<UiState>.and(in_state(AppState::InGame)),
            ),
        );

        app.add_systems(
            Update,
            hide_current_wave_finished_text
                .run_if(resource_removed::<NextWaveTimer>),
        );
    }
}
