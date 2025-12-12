use bevy::prelude::*;

#[derive(Default, Reflect, PartialEq, Debug, Component)]
pub enum EnemyState {
    #[default]
    Idle,
    /// The player is in the enemy fov and no obstacles are in the way, the enemy will be rotated
    /// towards the player over time until it has direct eye contact with the player
    PlayerInFOV,
    GoToAgentTarget,
    EnemyAgentReachedTarget,
    /// Enemy can see the player, will shoot the player now
    AttackPlayer,
    /// This state will be set when `enemy.health == 0.0`. A death animation will be played and
    /// afterwards the enemy will be despawned.
    Dead,
    /// This state is set when the enemies should rotate towards the player direction over time
    RotateTowardsPlayer,
}

impl EnemyState {
    pub fn update_state(&mut self, new_state: EnemyState) {
        if *self != new_state {
            info!("Enemy State change: {:?} -> {:?}", *self, new_state);
            *self = new_state;
        }
    }
}

/// A marker component which is spawned at locations the enemies patrol when they do not know where
/// the player is positioned.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EnemyHotspot;
