use bevy::prelude::*;

use crate::player::{
    camera::PlayerCameraPlugin, hud::PlayerHudPlugin,
    movement::PlayerMovementPlugin, shooting::PlayerShootingPlugin,
    spawn::PlayerSpawnPlugin,
};

pub mod camera;
mod hud;
mod movement;
pub mod shooting;
pub mod spawn;

#[derive(Component)]
pub struct Player {
    pub health: f32,
    pub state: PlayerState,
}

#[derive(Reflect, PartialEq)]
pub enum PlayerState {
    Idle,
    Walking,
    Running,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            health: 100.0,
            state: PlayerState::Idle,
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerSpawnPlugin)
            .add_plugins(PlayerCameraPlugin)
            .add_plugins(PlayerShootingPlugin)
            .add_plugins(PlayerHudPlugin)
            .add_plugins(PlayerMovementPlugin);
    }
}
