use std::f32::consts::FRAC_PI_2;

use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};

use crate::player::{
    camera::components::{PlayerCamera, PlayerCameraMode},
    components::Player,
};

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

pub fn camera_orbit_player(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut player_transform: Single<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera_transform: Single<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    let delta = mouse_motion.delta;

    if delta != Vec2::ZERO {
        // pitch like nodding yes with your head
        let delta_pitch = -delta.y * 0.002;

        // yaw like nodding no with your head
        let delta_yaw = -delta.x * 0.003;

        // existing rotation
        let (current_yaw, current_pitch, current_roll) =
            player_transform.rotation.to_euler(EulerRot::YXZ);

        let new_yaw = delta_yaw + current_yaw;

        let new_pitch = (delta_pitch + current_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        player_transform.rotation =
            Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, current_roll);
        camera_transform.rotation =
            Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, current_roll);
    }
}

pub fn camera_follow_player(
    mut camera_transform: Single<
        (&mut Transform, &PlayerCamera),
        (With<PlayerCamera>, Without<Player>),
    >,
    player_transform: Single<&Transform, (With<Player>, Without<PlayerCamera>)>,
) {
    camera_transform.0.translation = player_transform.translation;

    // increase y a bit so camera is more like at head of player
    camera_transform.0.translation.y += 0.3;

    if camera_transform.1.mode == PlayerCameraMode::ThirdPerson {
        camera_transform.0.translation.z += 1.5;
    }
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
