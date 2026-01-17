use bevy::prelude::*;

use crate::enemy::ai::{
    components::EnemyHotspot,
    messages::UpdateEnemyAgentTargetMessage,
    systems::{
        check_if_enemy_agent_reached_target, enemy_state_decision_system,
        handle_chasing_enemies, handle_set_new_enemy_agent_target_message,
        rotate_enemies_towards_player_over_time,
        update_enemy_agents_velocity_from_physics_velocity,
    },
};

pub mod components;
mod messages;
mod systems;

pub const ENEMY_VISION_RANGE: f32 = 30.0;
pub const ENEMY_FOV: f32 = 70.0;

// OUTDATED
// Enemy AI:
// 1. Enemy gets spawned (State idle)
// 2. Check with raycast whether player can be seen
// If yes: (Set state to AttackPlayer)
//     Shoot the player
// Else: (Set state to ChasingPlayer)
//     Get the current location of the player
//     Go to it via agent from landmass
//     When target reached, set EnemyState::CheckIfPlayerSeeable
// Repeat at step 2

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
            .add_systems(
                Update,
                (
                    update_enemy_agents_velocity_from_physics_velocity,
                    enemy_state_decision_system,
                    handle_chasing_enemies,
                    check_if_enemy_agent_reached_target,
                    rotate_enemies_towards_player_over_time,
                    handle_set_new_enemy_agent_target_message,
                ),
            );
    }
}
