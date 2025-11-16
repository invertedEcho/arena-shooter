use bevy::prelude::*;

use crate::{
    enemy::{
        ai::systems::{
            check_if_enemy_can_see_player, check_if_enemy_reached_target,
            handle_chasing_enemies,
        },
        shooting::systems::enemy_shoot_player,
    },
    game_flow::states::InGameState,
};

mod systems;

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
