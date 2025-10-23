use bevy::prelude::*;

use crate::{
    enemy::ai::systems::{
        apply_gravity_over_time, check_if_enemy_can_see_player,
        check_if_enemy_reached_target, move_enemy_with_agent_velocity,
        update_enemy_on_ground,
    },
    game_flow::states::AppState,
};

mod systems;

/// Enemy AI:
/// 1. Enemy gets spawned (State idle)
/// 2. Check with raycast whether player can be seen
/// If yes: (Set state to AttackPlayer)
///   Shoot the player
/// Else: (Set state to ChasingPlayer)
///   Get the current location of the player
///   Go to it via agent from landmass
/// Repeat at step 2

pub struct EnemyAiPlugin;

impl Plugin for EnemyAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                move_enemy_with_agent_velocity,
                check_if_enemy_can_see_player,
                update_enemy_on_ground,
                apply_gravity_over_time,
                check_if_enemy_reached_target,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Default, Reflect, PartialEq, Debug)]
pub enum EnemyState {
    #[default]
    Idle,
    /// Check if the enemy can see the player
    CheckIfPlayerSeeable,
    /// Going to the location of the player
    ChasingPlayer,
    /// Enemy can see the player, will shoot the player now
    AttackPlayer,
    /// This state will be set when `enemy.health == 0.0`. A death animation will be played and
    /// afterwards the enemy will be despawned.
    Dead,
}
