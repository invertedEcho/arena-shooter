use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::{
    Predicted,
    input::native::{ActionState, InputMarker},
};
use shared::protocol::{Inputs, Movement};

pub fn buffer_input(
    mut query: Query<&mut ActionState<Inputs>, With<InputMarker<Inputs>>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut action_state) = query.single_mut() {
        let mut movement = Movement::default();

        if keyboard_input.pressed(KeyCode::KeyW) {
            movement.forward = true;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            movement.left = true;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            movement.backwards = true;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            movement.right = true;
        }

        // FIXME: add
        // if keyboard_input.just_pressed(KeyCode::Space) {
        //     movement.jump = true;
        // }
        action_state.0 = Inputs::Movement(movement);
    }
}

pub fn handle_client_input(
    query: Query<
        (
            Entity,
            &Transform,
            &ActionState<Inputs>,
            &mut LinearVelocity,
        ),
        With<Predicted>,
    >,
    mut spatial_query: SpatialQuery,
) {
    for (entity, transform, res, mut velocity) in query {
        info!(
            "Movement on client, position {:?} | current velocity {:?}",
            transform.translation, velocity
        );
        shared::character_controller::systems::shared_movement(
            &mut velocity,
            res,
            &mut spatial_query,
            transform,
            [entity].to_vec(),
        );
    }
}

// TODO: Also send the inputs to the server
// pub fn handle_keyboard_input_for_player(
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     mut movement_action_writer: MessageWriter<MovementAction>,
//     player_query: Single<(Entity, &PlayerWeapons, &PlayerCameraState)>,
//     camera_transform: Single<&Transform, With<WorldCamera>>,
// ) {
//     let (player_entity, player_weapons, player_camera_state) =
//         player_query.into_inner();
//
//     if *player_camera_state == PlayerCameraState::FreeCam {
//         return;
//     }
//
//     let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft);
//     let reloading = player_weapons.reloading;
//
//     let speed = if shift_pressed && !reloading {
//         RUN_VELOCITY
//     } else {
//         WALK_VELOCITY
//     };
//
//     let forward_camera = camera_transform.forward();
//     let right = camera_transform.right();
//
//     let Ok(forward_camera) =
//         Dir3::from_xyz(forward_camera.x, 0.0, forward_camera.z)
//     else {
//         return;
//     };
//     let Ok(right) = Dir3::from_xyz(right.x, 0.0, right.z) else {
//         return;
//     };
//
//     let mut velocity = Vec3::ZERO;
//
//     if keyboard_input.pressed(KeyCode::KeyW) {
//         velocity += forward_camera * speed;
//     }
//     if keyboard_input.pressed(KeyCode::KeyA) {
//         velocity -= right * speed;
//     }
//     if keyboard_input.pressed(KeyCode::KeyD) {
//         velocity += right * speed;
//     }
//     if keyboard_input.pressed(KeyCode::KeyS) {
//         velocity -= forward_camera * speed;
//     }
//
//     if keyboard_input.just_pressed(KeyCode::Space) {
//         movement_action_writer.write(MovementAction {
//             direction: MovementDirection::Jump,
//             character_controller_entity: player_entity,
//         });
//     }
//
//     if velocity == Vec3::ZERO {
//         return;
//     }
//
//     movement_action_writer.write(MovementAction {
//         direction: MovementDirection::Move(velocity),
//         character_controller_entity: player_entity,
//     });
// }
