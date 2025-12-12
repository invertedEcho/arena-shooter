use bevy::prelude::*;

use crate::{
    player::{
        animate::PlayerAnimatePlugin,
        camera::{PlayerCameraPlugin, components::PlayerCameraState},
        hud::PlayerHudPlugin,
        shooting::PlayerShootingPlugin,
        spawn::PlayerSpawnPlugin,
    },
    shared::components::Health,
};

mod animate;
pub mod camera;
mod hud;
pub mod shooting;
pub mod spawn;

pub const DEFAULT_PLAYER_HEALTH: f32 = 100.0;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    health: Health,
    player_camera_state: PlayerCameraState,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            health: Health(DEFAULT_PLAYER_HEALTH),
            player_camera_state: PlayerCameraState::default(),
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
