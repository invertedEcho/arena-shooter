use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const DEFAULT_PLAYER_HEALTH: f32 = 100.0;

#[derive(Component, Debug, Reflect, Serialize, PartialEq, Deserialize)]
#[reflect(Component)]
pub struct Player;

/// This component marks an entity as ready to be used for exterrnal systems that depend on the player, such as the HUD
#[derive(Component)]
pub struct PlayerReady;

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    health: Health,
    aim_type: AimType,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            health: Health(DEFAULT_PLAYER_HEALTH),
            aim_type: AimType::Normal,
        }
    }
}

#[derive(Component)]
pub struct Health(pub f32);

// FIXME: camera stuff should not live in shared, move to client

#[derive(Component)]
pub struct PlayerWeaponModel;

#[derive(Component)]
pub struct FreeCam;

#[derive(Component)]
pub struct MuzzleFlash;

// TODO: rename me
#[derive(Component)]
pub struct InterpolateWeapon {
    pub target_position: Vec3,
}

#[derive(PartialEq, Clone, Component)]
pub enum AimType {
    Normal,
    Scoped,
}
