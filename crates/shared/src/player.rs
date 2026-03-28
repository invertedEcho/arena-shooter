use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    DEFAULT_HEALTH,
    components::Health,
    shooting::{
        AK47, GameWeapon, PlayerWeapon, PlayerWeapons, WeaponKind,
        WeaponSlotType, WeaponState, WeaponStats,
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
    player_cash: PlayerCash,
}

#[derive(Component, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PlayerState {
    pub shooting: bool,
    pub reloading: bool,
    pub dead: bool,
    pub active_weapon_slot: usize,
}

#[derive(Component)]
pub struct PlayerCash(pub usize);

pub const DEFAULT_PLAYER_WEAPONS: PlayerWeapons = PlayerWeapons {
    weapons: [
        PlayerWeapon {
            state: WeaponState {
                loaded_ammo: 30,
                carried_ammo: 120,
            },
            game_weapon: AK47,
        },
        PlayerWeapon {
            state: WeaponState {
                loaded_ammo: 15,
                carried_ammo: 50,
            },
            game_weapon: 
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
            player_cash: PlayerCash(0),
        }
    }
}

#[derive(Component, PartialEq, Clone, Debug)]
pub enum AimType {
    Normal,
    Scoped,
}
