use std::f32::consts::FRAC_PI_2;

use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};

use crate::player::components::Player;

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

// we only need to change transform for player as camera is a child of the player
pub fn camera_orbit_player(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut player_transform: Single<&mut Transform, With<Player>>,
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
    }
}
