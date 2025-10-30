use bevy::prelude::*;

use crate::{
    character_controller::{
        messages::MovementAction,
        systems::{
            apply_gravity_over_time, apply_movement_damping,
            handle_keyboard_input_for_player,
            handle_movement_actions_for_character_controllers,
            update_on_ground,
        },
    },
    game_flow::states::InGameState,
};

pub mod components;
pub mod messages;
mod systems;

pub const CHARACTER_CAPSULE_RADIUS: f32 = 0.2;
pub const CHARACTER_CAPSULE_LENGTH: f32 = 1.3;

// Remember to update update_agent_target when this is no longer negative.
pub const LOCAL_FEET_CHARACTER: f32 =
    -((CHARACTER_CAPSULE_LENGTH + CHARACTER_CAPSULE_RADIUS * 2.0) / 2.);

pub const WALK_VELOCITY: f32 = 2.0;
pub const RUN_VELOCITY: f32 = 5.0;
pub const JUMP_VELOCITY: f32 = 3.0;

pub const MAX_SLOPE_ANGLE: f32 = 45.0_f32.to_radians();

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MovementAction>()
            .add_systems(
                Update,
                (
                    update_on_ground,
                    apply_gravity_over_time,
                    handle_keyboard_input_for_player,
                    handle_movement_actions_for_character_controllers,
                )
                    .run_if(in_state(InGameState::Playing)),
            )
            .add_systems(Update, apply_movement_damping);
    }
}
