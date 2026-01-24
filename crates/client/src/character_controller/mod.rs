use bevy::prelude::*;
use shared::character_controller::messages::MovementAction;

use crate::{
    character_controller::systems::{
        apply_gravity_over_time, apply_movement_damping, check_above_head,
        handle_keyboard_input_for_player,
        handle_movement_actions_for_character_controllers, update_grounded,
    },
    game_flow::states::{AppState, InGameState},
};

pub mod components;
mod systems;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MovementAction>()
            .add_systems(
                FixedUpdate,
                (
                    apply_movement_damping,
                    update_grounded,
                    apply_gravity_over_time,
                    check_above_head.after(update_grounded),
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (
                    handle_keyboard_input_for_player,
                    handle_movement_actions_for_character_controllers,
                )
                    .run_if(in_state(InGameState::Playing)),
            );
    }
}
