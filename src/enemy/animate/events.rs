use bevy::prelude::*;

#[derive(Event)]
pub struct PlayEnemyDeathAnimationEvent(pub Entity);
