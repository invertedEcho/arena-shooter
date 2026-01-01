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
    player_camera_state: PlayerCameraState,
    aim_type: AimType,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            health: Health(DEFAULT_PLAYER_HEALTH),
            player_camera_state: PlayerCameraState::default(),
            aim_type: AimType::Normal,
        }
    }
}

#[derive(Component)]
pub struct Health(pub f32);

// FIXME: camera stuff should not live in shared, move to client

/// This camera is for rendering the whole world
/// It has RenderLayer 0
#[derive(Component)]
pub struct WorldCamera;

/// This camera is for rendering everything that should be above the world, so models dont clip
/// into walls for example. Right now it only holds the PlayerWeaponModel.
/// It has RenderLayer 1 so its rendered on top of WorldCamera
#[derive(Debug, Component, Default)]
pub struct ViewModelCamera;

#[derive(Component, Debug, Default, PartialEq, Reflect)]
pub enum PlayerCameraState {
    #[default]
    Normal,
    FreeCam,
}

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
