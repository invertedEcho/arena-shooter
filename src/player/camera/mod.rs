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
            update_yaw_pitch_on_mouse_motion, weapon_sway,
        },
    },
};

pub mod components;
pub mod messages;
mod systems;
pub mod weapon_positions;

pub const PLAYER_CAMERA_Y_OFFSET: f32 = 0.4;

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
                    weapon_sway,
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
