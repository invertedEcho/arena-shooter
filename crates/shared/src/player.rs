use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{DEFAULT_HEALTH, components::Health};

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
}

#[derive(Component, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PlayerState {
    pub shooting: bool,
    pub reloading: bool,
    pub dead: bool,
    pub active_weapon_slot: usize,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            health: Health(DEFAULT_HEALTH),
            aim_type: AimType::Normal,
            state: PlayerState::default(),
        }
    }
}

#[derive(Component, PartialEq, Clone, Debug)]
pub enum AimType {
    Normal,
    Scoped,
}
