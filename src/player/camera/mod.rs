use bevy::prelude::*;

use crate::{
    game_flow::states::InGameState,
    player::camera::{
        components::ViewModelCamera,
        messages::SpawnPlayerCamerasMessage,
        systems::{
            free_cam_orbit, handle_free_cam_movement, handle_player_scope_aim,
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

pub const DEFAULT_POSITION_PLAYER_WEAPON: Vec3 = Vec3 {
    x: 0.05,
    y: -0.14,
    z: -0.2,
};
pub const SCOPE_NEAR_POSITION_PLAYER_WEAPON: Vec3 = Vec3 {
    x: -0.04,
    y: -0.114,
    z: 0.1,
};

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnPlayerCamerasMessage>()
            .add_systems(
                Update,
                (update_yaw_pitch_on_mouse_motion, free_cam_orbit)
                    .run_if(in_state(InGameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    setup_player_cameras,
                    toggle_freecam,
                    handle_free_cam_movement,
                    handle_player_scope_aim,
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
