use bevy::prelude::*;

use crate::{
    game_flow::states::InGameState,
    player::camera::systems::{
        do_weapon_kickback, free_cam_orbit, handle_free_cam_movement,
        handle_player_scope_aim, handle_spawn_player_camera_message,
        interpolate_weapon_position, make_player_weapon_hidden,
        make_player_weapon_visible, setup_player_cameras, spawn_muzzle_flash,
        toggle_freecam, update_player_weapon_model,
        update_yaw_pitch_on_mouse_motion, weapon_sway,
    },
};

pub mod components;
mod systems;
pub mod weapon_positions;

pub const PLAYER_CAMERA_Y_OFFSET: f32 = 0.4;

#[derive(Message)]
pub struct SpawnPlayerCamera(pub Entity);

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnPlayerCamera>();
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
                handle_player_scope_aim,
                weapon_sway,
                update_player_weapon_model,
                spawn_muzzle_flash,
                interpolate_weapon_position,
                do_weapon_kickback,
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
