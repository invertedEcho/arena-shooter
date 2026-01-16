use crate::{
    enemy::shooting::{
        messages::EnemyKilledMessage,
        systems::{
            enemy_shoot_player, handle_enemy_killed_message,
            handle_player_bullet_hit_enemy_message,
            tick_enemy_shoot_player_cooldown_timer,
        },
    },
    game_flow::states::InGameState,
};
use bevy::prelude::*;

pub mod components;
pub mod messages;
pub mod systems;

pub struct EnemyShootingPlugin;

impl Plugin for EnemyShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EnemyKilledMessage>().add_systems(
            Update,
            (
                enemy_shoot_player,
                tick_enemy_shoot_player_cooldown_timer,
                handle_player_bullet_hit_enemy_message,
                handle_enemy_killed_message,
            )
                .run_if(in_state(InGameState::Playing)),
        );
    }
}
