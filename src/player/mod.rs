use bevy::prelude::*;

use crate::player::{
    animate::PlayerAnimatePlugin,
    camera::{PlayerCameraPlugin, components::PlayerCameraState},
    hud::PlayerHudPlugin,
    shooting::PlayerShootingPlugin,
    spawn::PlayerSpawnPlugin,
};

mod animate;
pub mod camera;
mod hud;
pub mod shooting;
pub mod spawn;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Player {
    pub health: f32,
    pub camera_state: PlayerCameraState,
}

pub const DEFAULT_PLAYER_HEALTH: f32 = 100.0;

impl Default for Player {
    fn default() -> Self {
        Player {
            health: DEFAULT_PLAYER_HEALTH,
            camera_state: PlayerCameraState::Normal,
        }
    }
}

#[derive(Message)]
pub struct PlayerDeathMessage;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_plugins(PlayerSpawnPlugin)
            .add_plugins(PlayerCameraPlugin)
            .add_plugins(PlayerShootingPlugin)
            .add_plugins(PlayerHudPlugin)
            .add_plugins(PlayerAnimatePlugin);
    }
}
