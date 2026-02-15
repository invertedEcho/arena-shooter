use bevy::prelude::*;

pub mod components;

pub const ENEMY_VISION_RANGE: f32 = 30.0;
pub const ENEMY_FOV: f32 = 70.0;

// TODO: move somewhere else
/// 0: The enemy entity that shot the player
#[derive(Message)]
pub struct EnemyShotPlayer(pub Entity);
