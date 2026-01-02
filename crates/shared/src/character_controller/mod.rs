use bevy::prelude::*;

use crate::character_controller::systems::{
    apply_gravity_over_time, apply_movement_damping, check_above_head,
    update_on_ground,
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
