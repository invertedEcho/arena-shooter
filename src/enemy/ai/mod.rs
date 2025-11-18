use bevy::prelude::*;

use crate::{
    enemy::ai::systems::{
        check_if_enemy_can_see_player, check_if_enemy_reached_target,
        handle_chasing_enemies,
    },
    game_flow::states::InGameState,
};

mod systems;

pub const ENEMY_VISION_RANGE: f32 = 30.0;
pub const ENEMY_FOV: f32 = 70.0;

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
// 1. Add shooting inaccuracy -> pick random x from 0 to 1 something like that
// 2. Add reaction time before firing, e.g. delay shooting after 0.2 - 0.6 after seeing the player
// 3. Randomize firing intervals, every 0.4 - 0.9 seconds
// 4. Simple movement (strafing to left or right) while shooting
// 5. Aim correction -> aim starts inaccurate, then gets more accurate over time
// 6. Enemies get alerted when player shoots -> maybe just make it that enemies patrol the map, but
//    if the player shoots, directly go to player location
// 7. investigation state -> when enemy cant see player anymore, go to last known location, and
//    just rotate the enemy for 4-6 seconds

pub struct EnemyAiPlugin;

impl Plugin for EnemyAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_chasing_enemies,
                check_if_enemy_can_see_player,
                check_if_enemy_reached_target,
            )
                .run_if(in_state(InGameState::Playing)),
        );
    }
}
