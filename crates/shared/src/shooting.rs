use std::fmt::Display;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// no idea if this number makes sense but works so far
pub const MAX_SHOOTING_DISTANCE: f32 = 200.0;

pub const DEFAULT_BULLET_DAMAGE: f32 = 7.5;

#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct PlayerWeapons {
    pub weapons: [PlayerWeapon; 2],
}

/// Static information of the weapon
#[derive(Serialize, Deserialize, PartialEq)]
pub struct GameWeapon {
    pub weapon_kind: WeaponKind,
    pub cost: usize,
    /// How much ammunition the weapon can hold at most (e.g. in the barrel)
    pub max_loaded_ammo: u64,
    pub slot_type: WeaponSlotType,
    pub damage: f32,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct PlayerWeapon {
    pub state: WeaponState,
    pub game_weapon: GameWeapon,
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
pub enum WeaponKind {
    Glock,
    AK47,
}

impl Display for WeaponKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeaponKind::Glock => write!(f, "Glock"),
            WeaponKind::AK47 => write!(f, "AK-47"),
        }
    }
}

pub const AK47: GameWeapon = GameWeapon {
    weapon_kind: WeaponKind::AK47,
    cost: 2000,
    max_loaded_ammo: 30,
    slot_type: WeaponSlotType::Primary,
    damage: 25.0,
};

pub const GLOCK: GameWeapon = GameWeapon {};

pub const ALL_GAME_WEAPONS: [GameWeapon; 2] = [
    AK47,
    GameWeapon {
        weapon_kind: WeaponKind::Glock,
        cost: 500,
    },
];
