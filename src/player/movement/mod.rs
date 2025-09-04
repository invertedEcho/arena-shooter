use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{ground_detection::components::GroundDetection, player::Player};

const PLAYER_WALK_SPEED: f32 = 2.0;
const PLAYER_RUN_SPEED: f32 = 5.0;

pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut LinearVelocity, &mut Transform, &GroundDetection), With<Player>>,
) {
    let (mut velocity, transform, ground_detection) = player.into_inner();

    let speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
        PLAYER_RUN_SPEED
    } else {
        PLAYER_WALK_SPEED
    };

    let mut local_velocity = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        local_velocity.z -= speed;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        local_velocity.x -= speed;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        local_velocity.x += speed;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        local_velocity.z += speed;
    }
    if keyboard_input.just_pressed(KeyCode::Space) && ground_detection.on_ground {
        velocity.y = 4.0;
    }

    let world_velocity = transform.rotation * local_velocity;
    velocity.x = world_velocity.x;
    velocity.z = world_velocity.z;
}
