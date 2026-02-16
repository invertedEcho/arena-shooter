use crate::{
    GameStateWave,
    enemy::shooting::{
        messages::EnemyKilledMessage,
        systems::{
            detect_killed_enemies, enemy_shoot_player,
            handle_enemy_killed_message,
            tick_enemy_shoot_player_cooldown_timer,
        },
    },
};
use bevy::prelude::*;
use shared::{GameStateServer, PlayerHitMessage};

pub mod components;
pub mod messages;
pub mod systems;

pub struct EnemyShootingPlugin;

impl Plugin for EnemyShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayerHitMessage>();
        app.add_message::<EnemyKilledMessage>().add_systems(
            Update,
            (
                enemy_shoot_player,
                tick_enemy_shoot_player_cooldown_timer,
                handle_enemy_killed_message,
                detect_killed_enemies,
            )
                .run_if(
                    resource_exists::<GameStateWave>
                        .and(in_state(GameStateServer::Running)),
                ),
        );
    }
}
