use bevy::prelude::*;
use lightyear::prelude::input::native::ActionState;

use crate::{
    character_controller::systems::{
        apply_gravity_over_time, apply_movement_damping, check_above_head,
        update_on_ground,
    },
    protocol::PlayerInputs,
};

pub mod components;
pub mod systems;

pub const CHARACTER_CAPSULE_RADIUS: f32 = 0.2;
pub const CHARACTER_CAPSULE_LENGTH: f32 = 1.3;
pub const CHARACTER_HEIGHT: f32 =
    CHARACTER_CAPSULE_LENGTH + CHARACTER_CAPSULE_RADIUS * 2.0;

pub const LOCAL_FEET_CHARACTER: f32 = -1.0;

pub const WALK_VELOCITY: f32 = 2.0;
pub const RUN_VELOCITY: f32 = 5.0;
pub const JUMP_VELOCITY: f32 = 3.0;

pub const MAX_SLOPE_ANGLE: f32 = 45.0_f32.to_radians();

pub const GROUND_CASTER_MAX_DISTANCE: f32 = 0.1;

pub const MAX_DISTANCE_SHAPE_CAST_MOVEMENT: f32 = 0.3;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                apply_movement_damping,
                update_on_ground,
                apply_gravity_over_time,
                check_above_head.after(update_on_ground),
            ),
        );
    }
}

pub fn convert_action_state_input_to_desired_velocity(
    action_state_input: &ActionState<PlayerInputs>,
) -> Vec3 {
    let speed = if action_state_input.run {
        RUN_VELOCITY
    } else {
        WALK_VELOCITY
    };

    let yaw = action_state_input.camera_yaw;
    let pitch = action_state_input.camera_pitch;
    // info!("\n");
    // info!("convert_action_state_input_to_desired_velocity;");
    // info!("direction: {:?}", action_state_input.direction);
    // info!("yaw {} | pitch {}", yaw, pitch);
    // info!("\n");

    let forward =
        Vec3::new(yaw.sin() * pitch.cos(), 0.0, yaw.cos() * pitch.cos())
            .normalize();

    let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin()).normalize();

    info!("forward camera: {}", forward);
    info!("right camera: {}", right);

    let mut desired_velocity = Vec3::ZERO;

    if action_state_input.direction.forward {
        desired_velocity -= forward * speed;
    }
    if action_state_input.direction.left {
        desired_velocity -= right * speed;
    }
    if action_state_input.direction.right {
        desired_velocity += right * speed;
    }
    if action_state_input.direction.backwards {
        desired_velocity += forward * speed;
    }

    desired_velocity
}
