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
    // TODO: maybe move this to player camera component
    pub current_walk_animation_step_index: usize,
    pub walking: bool,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            health: 100.0,
            current_walk_animation_step_index: 0,
            walking: true,
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
