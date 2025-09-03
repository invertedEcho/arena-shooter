use std::f32::consts::FRAC_PI_2;

use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};

use crate::player::{
    camera::components::{PlayerCamera, PlayerCameraMode},
    components::Player,
};

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

pub fn camera_orbit_player(
    mouse_motion: Res<AccumulatedMouseMotion>,
    player_camera: Single<&PlayerCamera>,
    mut player_transform: Single<&mut Transform, With<Player>>,
) {
    if !player_camera.mouse_motion_enabled {
        return;
    }

    let delta = mouse_motion.delta;

    if delta != Vec2::ZERO {
        // pitch like nodding yes with your head
        let delta_pitch = -delta.y * 0.001;

        // yaw like nodding no with your head
        let delta_yaw = -delta.x * 0.002;

        // existing rotation
        let (current_yaw, current_pitch, current_roll) =
            player_transform.rotation.to_euler(EulerRot::YXZ);

        let new_yaw = delta_yaw + current_yaw;

        let new_pitch = (delta_pitch + current_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        player_transform.rotation =
            Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, current_roll);
    }
}

pub fn update_player_camera_distance(
    mut camera_transform: Single<(&mut Transform, &PlayerCamera), Changed<PlayerCamera>>,
) {
    if camera_transform.1.mode == PlayerCameraMode::ThirdPerson {
        camera_transform.0.translation.z += 3.0;
    } else if camera_transform.1.mode == PlayerCameraMode::FirstPerson {
        if camera_transform.0.translation.z == 0.0 {
            return;
        }
        camera_transform.0.translation.z -= 3.0;
    }

    // TODO: add again
    // increase y a bit so camera is more like at head of player
    // camera_transform.0.translation.y += 0.3;
}

pub fn switch_between_first_and_third_person(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Single<&mut PlayerCamera>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyV) {
        match query.mode {
            PlayerCameraMode::FirstPerson => {
                query.mode = PlayerCameraMode::ThirdPerson;
            }
            PlayerCameraMode::ThirdPerson => {
                query.mode = PlayerCameraMode::FirstPerson;
            }
        }
    }
}

pub fn change_mouse_motion_enabled(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Single<&mut PlayerCamera>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        query.mouse_motion_enabled = !query.mouse_motion_enabled;
    }
}
