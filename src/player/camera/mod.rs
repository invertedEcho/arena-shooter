use bevy::prelude::*;

use crate::{
    game_flow::states::{AppState, InGameState},
    player::camera::{
        components::ViewModelCamera,
        messages::SpawnPlayerCamerasMessage,
        systems::{
            free_cam_orbit, handle_free_cam_movement,
            make_player_weapon_hidden, make_player_weapon_visible,
            setup_player_cameras, toggle_freecam,
            update_yaw_pitch_on_mouse_motion,
        },
    },
};

pub mod components;
pub mod messages;
mod systems;

pub const PLAYER_CAMERA_Y_OFFSET: f32 = 0.4;

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnPlayerCamerasMessage>()
            .add_systems(
                Update,
                (
                    update_yaw_pitch_on_mouse_motion.run_if(
                        in_state(AppState::InGame)
                            .and(in_state(InGameState::Playing)),
                    ),
                    setup_player_cameras,
                    toggle_freecam,
                    handle_free_cam_movement,
                    free_cam_orbit,
                ),
            )
            .add_systems(
                OnEnter(InGameState::Playing),
                make_player_weapon_visible,
            )
            .add_systems(
                OnExit(InGameState::Playing),
                make_player_weapon_hidden,
            );
    }
}
