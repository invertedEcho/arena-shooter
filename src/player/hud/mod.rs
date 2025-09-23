use bevy::prelude::*;

use crate::{
    game_flow::GameState,
    player::hud::systems::{
        spawn_bullet_hit_crosshair, spawn_player_hud, spawn_score_hud,
        update_player_ammo_text, update_player_health_text, update_score_hud,
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
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            OnEnter(GameState::InGame),
            (spawn_player_hud, spawn_score_hud),
        );
    }
}
