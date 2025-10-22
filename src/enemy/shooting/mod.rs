use crate::{
    enemy::shooting::{
        messages::EnemyKilledMessage,
        systems::{
            detect_player_bullet_collision_with_enemy,
            handle_enemy_killed_message,
            tick_enemy_shoot_player_cooldown_timer,
        },
    },
    game_flow::states::InGameState,
};
use bevy::prelude::*;

pub mod components;
pub mod messages;
mod systems;

pub struct EnemyShootingPlugin;

impl Plugin for EnemyShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EnemyKilledMessage>().add_systems(
            Update,
            (
                // enemy_shoot_player,
                tick_enemy_shoot_player_cooldown_timer,
                detect_player_bullet_collision_with_enemy,
                handle_enemy_killed_message,
            )
                .run_if(in_state(InGameState::Playing)),
        );
    }
}
