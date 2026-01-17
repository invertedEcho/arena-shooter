use bevy::prelude::*;

/// A marker component which is spawned at locations the enemies patrol when they do not know where
/// the player is positioned.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EnemyHotspot;
