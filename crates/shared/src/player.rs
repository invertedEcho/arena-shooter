use bevy::prelude::*;
use netvy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    DEFAULT_HEALTH,
    components::Health,
    shooting::{GameWeapon, WEAPON_AK47, WEAPON_GLOCK, WeaponState},
};

#[derive(Component, Debug, Reflect, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Player;

/// This component marks our player as ready to be used for external systems that depend on specific components being present, such as the HUD
#[derive(Component)]
pub struct OurPlayerReady;

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    health: Health,
    aim_type: AimType,
    state: PlayerState,
    weapons: PlayerWeapons,
    player_cash: PlayerCash,
}

#[derive(Component, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PlayerState {
    pub shooting: bool,
    pub reloading: bool,
    pub dead: bool,
}

#[derive(Component)]
pub struct PlayerCash(pub usize);

pub const DEFAULT_PLAYER_WEAPONS: PlayerWeapons = PlayerWeapons {
    active_weapon_slot: 0,
    weapons: [
        PlayerWeapon {
            state: WeaponState {
                loaded_ammo: 30,
                carried_ammo: 120,
            },
            game_weapon: WEAPON_AK47,
        },
        PlayerWeapon {
            state: WeaponState {
                loaded_ammo: 15,
                carried_ammo: 50,
            },
            game_weapon: WEAPON_GLOCK,
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

/// server sends this to all clients whenever a player kills another player so they visually hide the killed
/// player.
#[derive(Message, Serialize, Deserialize)]
pub struct PlayerKilled {
    pub player_killed: NetEntityId,
}

/// server sends this to all clients whenever the server respawned a player.
/// all clients will then make that player visible again, as they hid that player after receiving PlayerKilled message.
#[derive(Message, Serialize, Deserialize)]
pub struct PlayerRespawned {
    pub player_respawned: NetEntityId,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PlayerWeapon {
    pub state: WeaponState,
    pub game_weapon: GameWeapon,
}

#[derive(Component, Serialize, Deserialize, PartialEq, Debug)]
pub struct PlayerWeapons {
    pub weapons: [PlayerWeapon; 2],
    pub active_weapon_slot: usize,
}
