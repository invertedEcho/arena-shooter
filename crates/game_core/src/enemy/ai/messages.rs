use bevy::prelude::*;

/// This message is written whenever the enemy target should be updated. The agent target will be
/// set to the player's current location
#[derive(Message)]
pub struct UpdateEnemyAgentTargetMessage(pub Entity);

/// This message is written whenever any player hits an enemy
#[derive(Message)]
pub struct PlayerHitEnemy {
    /// The player that hit the enemy
    pub player_entity: Entity,
    /// The enemy that was hit
    pub enemy_entity: Entity,
}
