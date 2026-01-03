use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::{
    Predicted,
    input::native::{ActionState, InputMarker},
};
use shared::{
    character_controller::{
        components::Grounded, convert_action_state_input_to_desired_velocity,
    },
    protocol::PlayerInputs,
};

use crate::player::camera::components::PlayerCameraState;

/// This system reads input from the client and writes it to ActionState
pub fn buffer_input(
    mut action_state_input: Single<
        &mut ActionState<PlayerInputs>,
        With<InputMarker<PlayerInputs>>,
    >,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let direction = &mut action_state_input.0.direction;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.forward = true;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.left = true;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.backwards = true;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.right = true;
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        action_state_input.0.jump = true;
    }

    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        action_state_input.0.run = true;
    }
}

/// This system reads the input from the player, converts it into a desired_velocty and calls shared_movement with it
pub fn handle_client_movement(
    query: Query<
        (
            Entity,
            &Transform,
            &ActionState<PlayerInputs>,
            &mut LinearVelocity,
            &PlayerCameraState,
            &Grounded,
        ),
        With<Predicted>,
    >,
    mut spatial_query: SpatialQuery,
    // player_weapons: Single<&PlayerWeapons>,
) {
    for (
        entity,
        transform,
        player_input,
        mut velocity,
        player_camera_state,
        grounded,
    ) in query
    {
        info!(
            "Movement on client, position {:?} | current velocity {:?}",
            transform.translation, velocity
        );

        if *player_camera_state == PlayerCameraState::FreeCam {
            return;
        }

        // FIXME: REINTRODUCE
        // let reloading = player_weapons.reloading;

        let desired_velocity =
            convert_action_state_input_to_desired_velocity(player_input);

        shared::character_controller::systems::shared_movement(
            &mut velocity,
            desired_velocity,
            &mut spatial_query,
            transform,
            [entity].to_vec(),
            grounded.0,
        );
    }
}
