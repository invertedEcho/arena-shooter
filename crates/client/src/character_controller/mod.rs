use bevy::prelude::*;

use crate::{
    character_controller::{
        messages::MovementAction,
        systems::{
            apply_gravity_over_time, apply_movement_damping, check_above_head,
            handle_keyboard_input_for_player,
            handle_movement_actions_for_character_controllers, update_grounded,
            zero_player_velocity,
        },
    },
    game_flow::states::{AppState, InGameState},
};

pub mod components;
mod messages;
mod systems;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MovementAction>()
            .add_systems(
                FixedUpdate,
                (
                    update_grounded,
                    apply_gravity_over_time,
                    check_above_head.after(update_grounded),
                    handle_movement_actions_for_character_controllers,
                    apply_movement_damping,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (handle_keyboard_input_for_player,)
                    .run_if(in_state(InGameState::Playing)),
            );
        app.add_systems(OnExit(InGameState::Playing), zero_player_velocity);
    }
}
