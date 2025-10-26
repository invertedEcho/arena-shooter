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

#[derive(Component, Debug)]
pub struct Player {
    pub health: f32,
    // TODO: get rid of me or move me somewhere else
    pub camera_state: PlayerCameraState,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            health: 100.0,
            camera_state: PlayerCameraState::Normal,
        }
    }
}

#[derive(Message)]
pub struct PlayerDeathMessage;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerSpawnPlugin)
            .add_plugins(PlayerCameraPlugin)
            .add_plugins(PlayerShootingPlugin)
            .add_plugins(PlayerHudPlugin)
            .add_plugins(PlayerAnimatePlugin);
    }
}
