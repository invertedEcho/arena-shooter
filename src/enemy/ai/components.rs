use bevy::prelude::*;

#[derive(Default, Reflect, PartialEq, Debug)]
pub enum EnemyState {
    /// Go to hotspot markers on the map, used when Enemy has no information about the player and
    /// also not in the FOV
    PatrolHotspots,
    /// Check if the enemy can see the player, respecting a FOV with a cone
    #[default]
    CheckIfPlayerSeeable,
    /// Going to the last known location of the player
    GoToLastKnownLocation,
    /// Enemy can see the player, will shoot the player now
    AttackPlayer,
    /// This state will be set when `enemy.health == 0.0`. A death animation will be played and
    /// afterwards the enemy will be despawned.
    Dead,
}

/// A marker component which is spawned at locations the enemies patrol when they do not know where
/// the player is positioned.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EnemyHotspot;
