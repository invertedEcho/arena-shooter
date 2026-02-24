use bevy::prelude::*;

use crate::shared::{WeaponState, WeaponStats};

#[derive(Component)]
pub struct PlayerShootCooldownTimer(pub Timer);

#[derive(Component)]
pub struct PlayerWeapons {
    pub weapons: [Weapon; 2],
}

pub struct Weapon {
    pub stats: WeaponStats,
    pub state: WeaponState,
}
