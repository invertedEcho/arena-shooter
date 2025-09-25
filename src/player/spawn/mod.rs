use bevy::prelude::*;

use crate::player::spawn::components::PlayerSpawnLocation;

pub mod components;

pub struct PlayerSpawnPlugin;

impl Plugin for PlayerSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerSpawnLocation>();
    }
}

pub const PLAYER_CAPSULE_RADIUS: f32 = 0.2;
pub const PLAYER_CAPSULE_LENGTH: f32 = 1.0;
