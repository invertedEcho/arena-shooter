use bevy::prelude::*;

/// 0: The enemy entity that was killed
#[derive(Message)]
pub struct EnemyKilledMessage(pub Entity);
