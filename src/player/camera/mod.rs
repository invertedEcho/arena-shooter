use bevy::prelude::*;

use crate::{
    game_flow::GameState,
    player::camera::{
        components::PlayerCamera,
        systems::{
            camera_orbit_player, change_mouse_motion_enabled,
            setup_player_camera, switch_between_first_and_third_person,
            update_player_camera_distance,
        },
    },
};

pub mod components;
mod systems;

pub const PLAYER_CAMERA_Y_OFFSET: f32 = 0.3;

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                camera_orbit_player.run_if(in_state(GameState::InGame)),
                update_player_camera_distance,
                switch_between_first_and_third_person,
                change_mouse_motion_enabled,
                setup_player_camera,
            ),
        )
        .register_type::<PlayerCamera>();
    }
}
