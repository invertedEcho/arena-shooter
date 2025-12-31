use bevy::prelude::*;

#[derive(Message)]
pub struct EnemyKilledMessage(pub Entity);
