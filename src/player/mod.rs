use bevy::prelude::*;

use crate::player::{
    camera::PlayerCameraPlugin, movement::player_movement,
    shooting::PlayerShootingPlugin,
};

pub mod camera;
mod hud;
mod movement;
mod shooting;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player {
    health: u32,
}

impl Default for Player {
    fn default() -> Self {
        Player { health: 100 }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_plugins(PlayerCameraPlugin)
            .add_plugins(PlayerShootingPlugin)
            .add_systems(Update, (player_movement,));
    }
}
