use bevy::prelude::*;

use crate::player::{
    animate::PlayerAnimatePlugin,
    camera::{PlayerCameraPlugin, components::PlayerCameraState},
    hud::PlayerHudPlugin,
    movement::PlayerMovementPlugin,
    shooting::PlayerShootingPlugin,
    spawn::PlayerSpawnPlugin,
};

mod animate;
pub mod camera;
mod hud;
mod movement;
pub mod shooting;
pub mod spawn;

#[derive(Component, Debug)]
pub struct Player {
    pub health: f32,
    pub on_ground: bool,
    // TODO: get rid of me or move me somewhere else
    pub camera_state: PlayerCameraState,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            health: 100.0,
            on_ground: true,
            camera_state: PlayerCameraState::Normal,
        }
    }
}

#[derive(Event)]
pub struct PlayerDeathEvent;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerSpawnPlugin)
            .add_plugins(PlayerCameraPlugin)
            .add_plugins(PlayerShootingPlugin)
            .add_plugins(PlayerHudPlugin)
            .add_plugins(PlayerMovementPlugin)
            .add_plugins(PlayerAnimatePlugin);
    }
}
