use bevy::prelude::*;

use crate::{
    game_flow::GameState,
    player::{
        camera::PlayerCameraPlugin, hud::PlayerHudPlugin,
        movement::player_movement, shooting::PlayerShootingPlugin,
    },
};

pub mod camera;
mod hud;
mod movement;
pub mod shooting;

#[derive(Component, Reflect)]
#[reflect(Component, Default)]
pub struct Player {
    // #[reflect(default)]
    pub health: f32,
    pub state: PlayerState,
}

#[derive(Reflect, PartialEq)]
enum PlayerState {
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
        app.register_type::<Player>()
            .add_plugins(PlayerCameraPlugin)
            .add_plugins(PlayerShootingPlugin)
            .add_plugins(PlayerHudPlugin)
            .add_systems(
                Update,
                (player_movement).run_if(in_state(GameState::InGame)),
            );
    }
}
