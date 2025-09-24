use bevy::prelude::*;

#[derive(Event)]
pub struct EnemyKilledEvent(pub Entity);
