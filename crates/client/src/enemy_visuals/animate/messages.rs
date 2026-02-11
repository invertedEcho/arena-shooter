use bevy::prelude::*;

use crate::enemy_visuals::animate::EnemyAnimationType;

#[derive(Message)]
pub struct PlayEnemyAnimationMessage {
    /// Which enemy entity to play the animation for
    pub enemy: Entity,
    pub animation_type: EnemyAnimationType,
    /// Whether the animation should repeat
    pub repeat: bool,
}
