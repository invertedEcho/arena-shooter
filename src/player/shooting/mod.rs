use bevy::prelude::*;

use crate::{
    game_flow::GameState,
    player::shooting::systems::{
        basic_shooting, detect_enemy_bullet_collision_with_player,
        handle_blood_screen_effect, reload_player_weapon,
        tick_player_weapon_timer,
    },
};

pub mod components;
mod systems;

pub struct PlayerShootingPlugin;

impl Plugin for PlayerShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                basic_shooting,
                tick_player_weapon_timer,
                detect_enemy_bullet_collision_with_player,
                handle_blood_screen_effect,
                reload_player_weapon,
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}
