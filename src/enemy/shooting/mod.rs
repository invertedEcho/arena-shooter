use crate::enemy::shooting::{
    events::EnemyKilledEvent,
    systems::{
        detect_player_bullet_collision_with_enemy, enemy_shoot_player,
        handle_enemy_killed_event, tick_enemy_shoot_player_cooldown_timer,
    },
};
use bevy::prelude::*;

use crate::game_flow::AppState;

pub mod components;
mod events;
mod systems;

pub struct EnemyShootingPlugin;

impl Plugin for EnemyShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyKilledEvent>().add_systems(
            Update,
            (
                enemy_shoot_player,
                tick_enemy_shoot_player_cooldown_timer,
                detect_player_bullet_collision_with_enemy,
                handle_enemy_killed_event,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}
