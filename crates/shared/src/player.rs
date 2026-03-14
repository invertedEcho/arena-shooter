use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    DEFAULT_HEALTH,
    components::Health,
    shooting::{
        PlayerWeapons, Weapon, WeaponSlotType, WeaponState, WeaponStats,
        WeaponType,
    },
};

#[derive(Component, Debug, Reflect, Serialize, PartialEq, Deserialize)]
#[reflect(Component)]
pub struct Player;

/// This component marks an entity as ready to be used for external systems that depend on the player, such as the HUD
#[derive(Component)]
pub struct PlayerReady;

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    health: Health,
    aim_type: AimType,
    state: PlayerState,
    weapons: PlayerWeapons,
}

#[derive(Component, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PlayerState {
    pub shooting: bool,
    pub reloading: bool,
    pub dead: bool,
    pub active_weapon_slot: usize,
}

pub const DEFAULT_PLAYER_WEAPONS: PlayerWeapons = PlayerWeapons {
    weapons: [
        Weapon {
            stats: WeaponStats {
                weapon_type: WeaponType::AssaultRifle,
                max_loaded_ammo: 30,
                weapon_slot_type: WeaponSlotType::Primary,
            },
            state: WeaponState {
                loaded_ammo: 30,
                carried_ammo: 120,
            },
        },
        Weapon {
            stats: WeaponStats {
                weapon_type: WeaponType::Pistol,
                max_loaded_ammo: 15,
                weapon_slot_type: WeaponSlotType::Secondary,
            },
            state: WeaponState {
                loaded_ammo: 15,
                carried_ammo: 50,
            },
        },
    ],
};

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            health: Health(DEFAULT_HEALTH),
            aim_type: AimType::Normal,
            state: PlayerState::default(),
            weapons: DEFAULT_PLAYER_WEAPONS,
        }
    }
}

#[derive(Component, PartialEq, Clone, Debug)]
pub enum AimType {
    Normal,
    Scoped,
}
