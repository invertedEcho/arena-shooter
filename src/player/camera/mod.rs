use bevy::prelude::*;

use crate::{
    game_flow::GameState,
    player::camera::{
        components::PlayerCamera,
        systems::{
            camera_orbit_player, player_walk_animation, setup_player_camera,
        },
    },
};

pub mod components;
mod systems;

pub const PLAYER_CAMERA_Y_OFFSET: f32 = 0.4;

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                camera_orbit_player.run_if(in_state(GameState::InGame)),
                setup_player_camera,
                player_walk_animation,
            ),
        )
        .register_type::<PlayerCamera>();
    }
}
