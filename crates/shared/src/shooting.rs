use std::fmt::Display;

use bevy::prelude::*;
use netvy::prelude::*;
use serde::{Deserialize, Serialize};

// no idea if this number makes sense but works so far
pub const MAX_SHOOTING_DISTANCE: f32 = 200.0;

#[derive(Component, Serialize, Deserialize, PartialEq, Debug)]
pub struct PlayerWeapons {
    pub weapons: [PlayerWeapon; 2],
    pub active_weapon_slot: usize,
}

/// Static information of the weapon
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct GameWeapon {
    pub kind: WeaponKind,
    pub cost: usize,
    /// How much ammunition the weapon can hold at most (e.g. in the barrel)
    pub max_loaded_ammo: u64,
    pub slot_type: WeaponSlotType,
    pub damage: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PlayerWeapon {
    pub state: WeaponState,
    pub game_weapon: GameWeapon,
}

#[derive(Component, Serialize, Deserialize, PartialEq, Debug)]
pub struct WeaponState {
    pub loaded_ammo: u64,
    pub carried_ammo: u64,
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum WeaponSlotType {
    Primary,
    Secondary,
}

#[derive(Reflect, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum WeaponKind {
    Glock,
    AK47,
    P90,
    SniperRifle,
}

impl Display for WeaponKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeaponKind::Glock => write!(f, "Glock"),
            WeaponKind::AK47 => write!(f, "AK-47"),
            WeaponKind::P90 => write!(f, "P-90"),
            WeaponKind::SniperRifle => write!(f, "Sniper"),
        }
    }
}

pub const WEAPON_AK47: GameWeapon = GameWeapon {
    kind: WeaponKind::AK47,
    cost: 2000,
    max_loaded_ammo: 30,
    slot_type: WeaponSlotType::Primary,
    damage: 30.0,
};

pub const WEAPON_GLOCK: GameWeapon = GameWeapon {
    kind: WeaponKind::Glock,
    cost: 500,
    max_loaded_ammo: 15,
    slot_type: WeaponSlotType::Secondary,
    damage: 20.0,
};

pub const WEAPON_P90: GameWeapon = GameWeapon {
    kind: WeaponKind::P90,
    cost: 2500,
    max_loaded_ammo: 40,
    slot_type: WeaponSlotType::Primary,
    damage: 20.0,
};

pub const WEAPON_SNIPER_RIFLE: GameWeapon = GameWeapon {
    kind: WeaponKind::SniperRifle,
    cost: 3000,
    max_loaded_ammo: 5,
    slot_type: WeaponSlotType::Primary,
    damage: 100.0,
};

pub const ALL_GAME_WEAPONS: [GameWeapon; 4] =
    [WEAPON_AK47, WEAPON_GLOCK, WEAPON_P90, WEAPON_SNIPER_RIFLE];

pub fn get_game_weapon_by_kind(weapon_kind: &WeaponKind) -> GameWeapon {
    match weapon_kind {
        WeaponKind::Glock => WEAPON_GLOCK,
        WeaponKind::AK47 => WEAPON_AK47,
        WeaponKind::P90 => WEAPON_P90,
        WeaponKind::SniperRifle => WEAPON_SNIPER_RIFLE,
    }
}

/// server sends this to all clients whenever a player kills another player so they visually hide the killed
/// player.
#[derive(Message, Serialize, Deserialize)]
pub struct PlayerKilled {
    pub player_killed: NetEntityId,
}
