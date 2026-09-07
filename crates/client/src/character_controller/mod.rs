use bevy::prelude::*;
use shared::character_controller::collide_and_slide_system;

use crate::{
    character_controller::{
        messages::JumpAction,
        systems::{
            apply_gravity, apply_movement_damping, check_above_head,
            exclude_added_world_object_from_ground_caster, handle_jump_action,
            handle_keyboard_input_for_player, update_grounded,
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
        app.add_message::<JumpAction>()
            .add_systems(
                Update,
                (
                    update_grounded,
                    apply_gravity,
                    check_above_head.after(update_grounded),
                    handle_jump_action,
                    apply_movement_damping,
                    exclude_added_world_object_from_ground_caster,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(FixedUpdate, collide_and_slide_system)
            .add_systems(
                Update,
                (handle_keyboard_input_for_player,)
                    .run_if(in_state(InGameState::Playing)),
            );
        app.add_systems(OnExit(InGameState::Playing), zero_player_velocity);
    }
}
