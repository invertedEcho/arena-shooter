use bevy::prelude::*;

use crate::shared::{WeaponState, WeaponStats};

#[derive(Component)]
pub struct PlayerShootCooldownTimer(pub Timer);

// TODO: this stuff should probably also only be spawned on server and synced to clients...
#[derive(Component)]
pub struct PlayerWeapons {
    pub weapons: [Weapon; 2],
}

pub struct Weapon {
    pub stats: WeaponStats,
    pub state: WeaponState,
}

/// original_rotation gets updated whenever the player uses the mouse to look around.
/// This way, we can have a camera kickback effect when shooting, but also know, to what value we
/// need to slerp back to
#[derive(Component, Default)]
pub struct ShootRecoil {
    pub original_rotation: Quat,
}
