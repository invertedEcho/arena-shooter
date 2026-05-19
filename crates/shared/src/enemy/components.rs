use std::time::Instant;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::enemy::UPDATE_ENEMY_STATE_COOLDOWN_SECONDS;

/// A marker component for an enemy
#[derive(Component, Default, Serialize, Deserialize, PartialEq)]
pub struct Enemy;

#[derive(Component)]
pub struct EnemyLastStateUpdate(pub Instant);

#[derive(
    Default, PartialEq, Debug, Component, Serialize, Deserialize, Reflect,
)]
#[reflect(Component)]
pub enum EnemyState {
    #[default]
    Idle,
    /// The player is in the enemy fov and no obstacles are in the way, the enemy will be rotated
    /// towards the player over time until it has direct eye contact with the player
    PlayerInFOV,
    GoToAgentTarget,
    EnemyAgentReachedTarget,
    /// The enemy will attack the given player
    AttackPlayer(Entity),
    /// This state will be set when `enemy.health == 0.0`. A death animation will be played and
    /// afterwards the enemy will be despawned.
    Dead,
    /// This state is set when the enemies should rotate towards the given player direction over time
    RotateTowardsPlayer(Entity),
}

impl EnemyState {
    /// A wrapper method to update the state of the enemy to avoid unncesessary state change
    /// triggers, and avoiding too frequent state updates. It also ignores state update requests if
    /// the enemy is dead.
    ///
    /// Arguments:
    /// force_update: If this is true, any checks are skipped and the state update will always be
    /// applied.
    pub fn update_state(
        &mut self,
        new_state: EnemyState,
        last_state_update: &mut EnemyLastStateUpdate,
        force_update: bool,
    ) {
        if force_update {
            *self = new_state;
            return;
        }

        if *self == EnemyState::Dead {
            return;
        }

        let same_state = *self == new_state;
        let state_update_on_cooldown = last_state_update.0.elapsed().as_secs()
            < UPDATE_ENEMY_STATE_COOLDOWN_SECONDS;

        if !same_state && !state_update_on_cooldown {
            debug!("Enemy State change: {:?} -> {:?}", self, new_state);
            *self = new_state;
            last_state_update.0 = Instant::now();
        }
    }

    pub fn is_dead(&self) -> bool {
        *self == EnemyState::Dead
    }
}

/// This component marks an enemy as ready to be used for external systems
#[derive(Component)]
pub struct EnemyReady;
