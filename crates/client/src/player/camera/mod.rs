use bevy::prelude::*;

use crate::{
    game_flow::states::InGameState,
    player::camera::{
        messages::UpdatePlayerWeaponModel,
        systems::{
            free_cam_orbit, handle_free_cam_movement,
            handle_spawn_player_camera_message, interpolate_weapon_position,
            make_player_weapon_hidden, make_player_weapon_visible,
            recoil_camera_kickback, recoil_slerp_back, setup_player_cameras,
            spawn_muzzle_flash, toggle_freecam, update_player_weapon_model,
            update_yaw_pitch_on_mouse_motion, weapon_model_kickback,
            weapon_sway,
        },
    },
};

pub mod components;
pub mod messages;
mod systems;
pub mod weapon_positions;

pub const PLAYER_CAMERA_Y_OFFSET: f32 = 0.4;

#[derive(Message)]
pub struct SpawnPlayerCamera(pub Entity);

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnPlayerCamera>()
            .add_message::<UpdatePlayerWeaponModel>();
        app.add_systems(
            Update,
            (update_yaw_pitch_on_mouse_motion, free_cam_orbit)
                .run_if(in_state(InGameState::Playing)),
        )
        .add_systems(
            Update,
            (
                setup_player_cameras,
                handle_spawn_player_camera_message,
                handle_free_cam_movement,
                weapon_sway,
                update_player_weapon_model,
                spawn_muzzle_flash,
                interpolate_weapon_position,
                weapon_model_kickback,
                recoil_camera_kickback,
                recoil_slerp_back,
                toggle_freecam,
            ),
        )
        .add_systems(OnEnter(InGameState::Playing), make_player_weapon_visible)
        .add_systems(OnExit(InGameState::Playing), make_player_weapon_hidden);
    }
}

pub fn get_main_menu_camera_transform() -> Transform {
    Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y)
}
