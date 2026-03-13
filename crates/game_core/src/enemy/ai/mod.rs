use bevy::prelude::*;
use shared::GameStateServer;

use crate::enemy::ai::{
    messages::{PlayerHitEnemy, UpdateEnemyAgentTargetMessage},
    systems::{
        check_if_enemy_agent_reached_target, enemy_state_decision_system,
        handle_chasing_enemies, handle_set_new_enemy_agent_target_message,
        read_player_hit_enemy_messages, retry_get_new_agent_target,
        rotate_enemies_towards_player_over_time, update_enemy_agents_velocity,
        zero_enemy_velocity,
    },
};

pub mod components;
pub mod messages;
mod systems;

// Roadmap to realistic enemy AI:
// 1. Add shooting inaccuracy -> pick random x from 0 to 1 something like that -> DONE
// 2. Add reaction time before firing, e.g. delay shooting after 0.2 - 0.6 after seeing the player
// 3. Randomize firing intervals, every 0.4 - 0.9 seconds -> DONE
// 4. Simple movement (strafing to left or right) while shooting
// 5. Aim correction -> aim starts inaccurate, then gets more accurate over time
// 6. Enemies get alerted when player shoots -> maybe just make it that enemies patrol the map, but
//    if the player shoots, directly go to player location
// 7. investigation state -> when enemy cant see player anymore, go to last known location, and
//    just rotate the enemy for 4-6 seconds

pub struct EnemyAiPlugin;

impl Plugin for EnemyAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<UpdateEnemyAgentTargetMessage>()
            .add_message::<PlayerHitEnemy>();

        app.add_systems(
            FixedUpdate,
            (
                update_enemy_agents_velocity,
                enemy_state_decision_system,
                handle_chasing_enemies,
                check_if_enemy_agent_reached_target,
                rotate_enemies_towards_player_over_time,
                handle_set_new_enemy_agent_target_message,
                retry_get_new_agent_target,
                read_player_hit_enemy_messages,
            )
                .run_if(in_state(GameStateServer::Running)),
        );

        app.add_systems(OnEnter(GameStateServer::Paused), zero_enemy_velocity);
    }
}
