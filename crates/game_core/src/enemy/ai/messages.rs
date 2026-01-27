use bevy::prelude::*;

// This message is written whenever the enemy target should be updated. The agent target will be
// set to the player's current location
#[derive(Message)]
pub struct UpdateEnemyAgentTargetMessage(pub Entity);
