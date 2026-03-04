use std::fmt::Display;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// no idea if this number makes sense but works so far
pub const MAX_SHOOTING_DISTANCE: f32 = 200.0;

pub const DEFAULT_BULLET_DAMAGE: f32 = 7.5;

#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct PlayerWeapons {
    pub weapons: [Weapon; 2],
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Weapon {
    pub stats: WeaponStats,
    pub state: WeaponState,
}

/// Static information of the weapon
#[derive(Component, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeaponStats {
    /// The type of the weapon
    pub weapon_type: WeaponType,
    /// How much ammunition the weapon can hold at most (e.g. in the barrel)
    pub max_loaded_ammo: u64,
    pub weapon_slot_type: WeaponSlotType,
}

#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct WeaponState {
    pub loaded_ammo: u64,
    pub carried_ammo: u64,
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub enum WeaponSlotType {
    Primary,
    Secondary,
}

#[derive(Reflect, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum WeaponType {
    Pistol,
    AssaultRifle,
}

impl Display for WeaponType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeaponType::Pistol => write!(f, "Pistol"),
            WeaponType::AssaultRifle => write!(f, "Assault Rifle"),
        }
    }
}
