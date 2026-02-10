use bevy::prelude::*;

use crate::shared::{WeaponState, WeaponStats};

#[derive(Component)]
pub struct PlayerShootCooldownTimer(pub Timer);

#[derive(Component)]
pub struct PlayerWeapons {
    pub active_slot: usize,
    pub weapons: [Weapon; 2],
    pub shooting: bool,
    pub reloading: bool,
}

pub struct Weapon {
    pub stats: WeaponStats,
    pub state: WeaponState,
}
