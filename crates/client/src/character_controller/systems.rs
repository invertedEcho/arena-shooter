use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::Controlled;
use shared::{
    GRAVITY, Medkit,
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS, JUMP_VELOCITY,
        RUN_VELOCITY, WALK_VELOCITY, apply_collide_and_slide,
        components::{CharacterController, Grounded},
    },
};

use crate::{
    character_controller::messages::{MovementAction, MovementDirection},
    player::{
        camera::components::{PlayerCameraState, WorldCamera},
        shooting::components::PlayerWeapons,
    },
};

pub fn handle_keyboard_input_for_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut movement_action_writer: MessageWriter<MovementAction>,
    player_query: Single<(Entity, &PlayerWeapons, &PlayerCameraState)>,
    camera_transform: Single<&Transform, With<WorldCamera>>,
) {
    let (player_entity, player_weapons, player_camera_state) =
        player_query.into_inner();

    if *player_camera_state == PlayerCameraState::FreeCam {
        return;
    }

    let sprint = keyboard_input.pressed(KeyCode::ShiftLeft);
    let reloading = player_weapons.reloading;

    let speed = if sprint && !reloading {
        RUN_VELOCITY
    } else {
        WALK_VELOCITY
    };

    let forward_camera = camera_transform.forward();
    let right = camera_transform.right();

    let Ok(forward_camera) =
        Dir3::from_xyz(forward_camera.x, 0.0, forward_camera.z)
    else {
        return;
    };
    let Ok(right) = Dir3::from_xyz(right.x, 0.0, right.z) else {
        return;
    };

    let mut desired_velocity = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        desired_velocity += forward_camera * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        desired_velocity -= right * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        desired_velocity += right * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        desired_velocity -= forward_camera * speed;
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_action_writer.write(MovementAction {
            desired_velocity: MovementDirection::Jump,
            character_controller_entity: player_entity,
            sprinting: sprint,
        });
    }

    // we always send a movementaction, so move_towards will handle deceleration, because we move
    // towards zero velocity.
    movement_action_writer.write(MovementAction {
        desired_velocity: MovementDirection::Move(desired_velocity),
        character_controller_entity: player_entity,
        sprinting: sprint,
    });
}

pub fn handle_movement_actions_for_character_controllers(
    mut movement_action_reader: MessageReader<MovementAction>,
    mut character_controller_query: Query<
        (&mut LinearVelocity, &Grounded, &Transform),
        With<CharacterController>,
    >,
    mut spatial_query: SpatialQuery,
    time: Res<Time>,
    medkit_query: Query<Entity, With<Medkit>>,
) {
    for movement_action in movement_action_reader.read() {
        let sprinting = movement_action.sprinting;
        let direction = &movement_action.desired_velocity;
        let character_controller_entity =
            movement_action.character_controller_entity;
        let Ok((mut velocity, grounded, transform)) =
            character_controller_query.get_mut(character_controller_entity)
        else {
            warn!(
                "Failed to find Character Controller by Entity {}",
                character_controller_entity
            );
            continue;
        };

        match *direction {
            MovementDirection::Jump => {
                if grounded.0 {
                    velocity.y = JUMP_VELOCITY;
                }
            }
            MovementDirection::Move(desired_velocity) => {
                let max_delta = get_max_delta(desired_velocity, sprinting);
                let new_velocity = move_towards_vec(
                    velocity.0,
                    desired_velocity,
                    max_delta * time.delta_secs(),
                );

                velocity.x = new_velocity.x;
                velocity.z = new_velocity.z;

                // exclude medkits because we want to be able to walk through medkits
                let excluded_entities: Vec<Entity> = medkit_query
                    .iter()
                    .chain(std::iter::once(character_controller_entity))
                    .collect();

                let spatial_query_filter = &SpatialQueryFilter::default()
                    .with_excluded_entities(excluded_entities.clone());

                apply_collide_and_slide(
                    &mut velocity,
                    desired_velocity,
                    transform,
                    &mut spatial_query,
                    spatial_query_filter,
                    time.delta_secs(),
                    0,
                );
            }
        }
    }
}

fn get_max_delta(desired_velocity: Vec3, sprinting: bool) -> f32 {
    const DECELERATION: f32 = 5.0;
    const ACCELERATION: f32 = 11.0;

    if sprinting {
        if desired_velocity == Vec3::ZERO {
            DECELERATION * 2.0
        } else {
            ACCELERATION * 2.0
        }
    } else {
        if desired_velocity == Vec3::ZERO {
            DECELERATION
        } else {
            ACCELERATION
        }
    }
}

/// currrent_velocity: Our current velocity
/// target_velocity: Our target velocity, e.g. the max velocity
/// max_delta: how fast are we allowed to change per frame. with this, we can control, how fast we
///            accelerate or deccelerate
fn move_towards_vec(
    current_velocity: Vec3,
    target_velocity: Vec3,
    max_delta: f32,
) -> Vec3 {
    // the difference between our current velocity and the target velocity
    let delta = target_velocity - current_velocity;

    // remember, the length of the vector is the distance between origin and destination
    let distance = delta.length();

    if distance <= max_delta || distance == 0.0 {
        target_velocity
    } else {
        // to get normalized vector (which is only direction), we divide difference between our two
        // vectors with the distance of that vector
        let normalized_delta = delta / distance;

        // as max_delta says how much we are allowed to change velocity per frame, by multiplying with max_delta,
        // we get new vector, but only changing it as allowed by max_dellta
        // normalized vektor = length 1, "stretching" that vector but only upon max_delta.
        current_velocity + normalized_delta * max_delta
    }
}

/// Updates the [`Grounded`] component for character controllers.
pub fn update_grounded(
    mut query: Query<
        (&ShapeHits, &mut Grounded, &mut LinearVelocity),
        With<CharacterController>,
    >,
) {
    for (hits, mut grounded, mut velocity) in &mut query {
        let on_ground = !hits.0.is_empty();

        if grounded.0 != on_ground {
            grounded.0 = on_ground;
        }

        if on_ground && velocity.y < 0.0 {
            velocity.y = 0.0
        }
    }
}

pub fn apply_gravity_over_time(
    query: Query<(&Grounded, &mut LinearVelocity), With<CharacterController>>,
    time: Res<Time>,
) {
    for (grounded, mut velocity) in query {
        if !grounded.0 {
            velocity.y -= GRAVITY * time.delta_secs();
        }
    }
}

pub fn zero_player_velocity(
    mut player_velocity: Single<&mut LinearVelocity, With<Controlled>>,
) {
    player_velocity.x = 0.0;
    player_velocity.z = 0.0;
}

pub fn check_above_head(
    query: Query<
        (Entity, &mut LinearVelocity, &Transform, &Grounded),
        With<CharacterController>,
    >,
    spatial_query: SpatialQuery,
) {
    for (entity_itself, mut velocity, transform, grounded) in query {
        if grounded.0 {
            continue;
        };
        let Some(_) = spatial_query.cast_shape(
            &Collider::capsule(
                CHARACTER_CAPSULE_RADIUS,
                CHARACTER_CAPSULE_LENGTH,
            ),
            transform.translation,
            Quat::IDENTITY,
            Dir3::Y,
            // TODO: investigate whether we can further decrease this value
            &ShapeCastConfig::default().with_max_distance(0.1),
            &SpatialQueryFilter::default()
                .with_excluded_entities([entity_itself]),
        ) else {
            continue;
        };

        // if there is something above the current shape, stop vertical movement, to prevent
        // clipping into ceilings
        velocity.y -= 0.5;
    }
}
