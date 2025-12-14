use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    GRAVITY,
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS, JUMP_VELOCITY,
        MAX_SLOPE_ANGLE, RUN_VELOCITY, WALK_VELOCITY,
        components::{CharacterController, Grounded, MovementState},
        messages::{MovementAction, MovementDirection},
    },
    player::{
        camera::components::PlayerCameraState,
        shooting::components::PlayerWeapon,
    },
    world::world_objects::medkit::Medkit,
};

pub fn handle_keyboard_input_for_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut movement_action_writer: MessageWriter<MovementAction>,
    player_query: Single<(
        &Transform,
        &mut MovementState,
        Entity,
        &PlayerWeapon,
        &PlayerCameraState,
    )>,
) {
    let (
        player_transform,
        mut movement_state,
        player_entity,
        player_weapon,
        player_camera_state,
    ) = player_query.into_inner();

    if *player_camera_state == PlayerCameraState::FreeCam {
        return;
    }

    let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft);
    let reloading = player_weapon.reloading;

    let speed = if shift_pressed && !reloading {
        RUN_VELOCITY
    } else {
        WALK_VELOCITY
    };

    let mut local_velocity = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        local_velocity.z -= 1.0 * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        local_velocity.x -= 1.0 * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        local_velocity.x += 1.0 * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        local_velocity.z += 1.0 * speed;
    }

    if local_velocity.x == 0.0 && local_velocity.z == 0.0 {
        if *movement_state != MovementState::Idle {
            *movement_state = MovementState::Idle;
        }
    } else if speed == RUN_VELOCITY {
        if *movement_state != MovementState::Running {
            *movement_state = MovementState::Running;
        }
    } else if speed == WALK_VELOCITY
        && *movement_state != MovementState::Walking
    {
        *movement_state = MovementState::Walking;
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_action_writer.write(MovementAction {
            direction: MovementDirection::Jump,
            character_controller_entity: player_entity,
        });
    }

    if local_velocity == Vec3::ZERO {
        return;
    }

    let world_velocity = player_transform.rotation * local_velocity;

    movement_action_writer.write(MovementAction {
        direction: MovementDirection::Move(world_velocity),
        character_controller_entity: player_entity,
    });
}

pub fn handle_movement_actions_for_character_controllers(
    mut movement_action_reader: MessageReader<MovementAction>,
    mut character_controller_query: Query<
        (&mut LinearVelocity, &Grounded, &Transform),
        With<CharacterController>,
    >,
    spatial_query: SpatialQuery,
    time: Res<Time>,
    medkit_query: Query<Entity, With<Medkit>>,
) {
    for movement_action in movement_action_reader.read() {
        let direction = &movement_action.direction;
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
            // TODO: should probably move the content of this block elsewhere
            MovementDirection::Move(world_velocity) => {
                let Ok(direction_from_world_velocity) =
                    Dir3::new(world_velocity)
                else {
                    velocity.x = 0.0;
                    velocity.z = 0.0;
                    return;
                };

                let ray_origin = transform.translation
                    - direction_from_world_velocity.as_vec3() * 0.025;
                let max_distance = 0.3;

                let excluded_entities: Vec<Entity> = medkit_query
                    .iter()
                    .chain(std::iter::once(character_controller_entity))
                    .collect();

                // also exclude medkits
                let spatial_query_filter = &SpatialQueryFilter::default()
                    .with_excluded_entities(excluded_entities);

                if let Some(hit_ahead) = spatial_query.cast_shape(
                    &Collider::capsule(
                        CHARACTER_CAPSULE_RADIUS,
                        CHARACTER_CAPSULE_LENGTH,
                    ),
                    ray_origin,
                    transform.rotation,
                    direction_from_world_velocity,
                    &ShapeCastConfig {
                        max_distance,
                        ..default()
                    },
                    spatial_query_filter,
                ) {
                    // obstacle in the way, check if we can slimb it
                    // a normal is just a direction something is facing
                    let normal = hit_ahead.normal1;
                    let slope_angle = normal.angle_between(Vec3::Y);
                    let slope_climable = slope_angle < MAX_SLOPE_ANGLE;

                    if slope_climable {
                        debug!("MOVEMENT: Climable slope!");
                        // this is the most important part to make the slope climbing possible.
                        // instead of trying to go straight, we slide along the ground
                        velocity.0 =
                            world_velocity.reject_from_normalized(normal);

                        // slope snapping
                        let ray_down_origin =
                            transform.translation + Vec3::Y * 0.5;
                        let ray_down_direction = Dir3::NEG_Y;
                        let max_down_distance = 1.0;

                        if let Some(hit_down) = spatial_query.cast_ray(
                            ray_down_origin,
                            ray_down_direction,
                            max_down_distance,
                            true,
                            spatial_query_filter,
                        ) {
                            let hit_down_point = ray_down_origin
                                + ray_down_direction * hit_down.distance;
                            let hit_down_y = hit_down_point.y;
                            let player_y = transform.translation.y;
                            let difference_y = hit_down_y - player_y;
                            if difference_y.abs() < 0.3 {
                                debug!(
                                    "Snapping character controller to slope"
                                );
                                velocity.y = difference_y / time.delta_secs();
                            }
                        }
                    } else {
                        debug!(
                            "MOVEMENT: Obstacle in the way, sliding along wall"
                        );
                        // not climable, e.g. a wall. we want to slide along the wall, similar to the collide
                        // and slide algorithm
                        // the main difference is that we ignore the Y part, because its too step, so we dont
                        // want to climb up
                        let impulse =
                            world_velocity.reject_from_normalized(normal);
                        velocity.x = impulse.x;
                        velocity.z = impulse.z
                    }
                } else {
                    debug!("MOVEMENT: No obstacle ahead, free movement");
                    // no obstacle ahead, free movement
                    velocity.x = world_velocity.x;
                    velocity.z = world_velocity.z;
                }
            }
        }
    }
}

/// Updates the [`Grounded`] status for character controllers.
pub fn update_on_ground(
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

        if on_ground && velocity.y <= 0.0 {
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

// Apply damping in the XZ Plane, basically this is deceleration over time
pub fn apply_movement_damping(
    query: Query<&mut LinearVelocity, With<CharacterController>>,
) {
    for mut velocity in query {
        velocity.x *= 0.9;
        velocity.z *= 0.9;
    }
}
