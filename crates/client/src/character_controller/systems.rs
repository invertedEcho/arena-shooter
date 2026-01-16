use avian3d::prelude::*;
use bevy::prelude::*;
use shared::{
    GRAVITY,
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
    },
};

use crate::{
    character_controller::{
        JUMP_VELOCITY, MAX_SLOPE_ANGLE, RUN_VELOCITY, WALK_VELOCITY,
        components::{CharacterController, Grounded},
        messages::{MovementAction, MovementDirection},
    },
    player::{
        camera::components::{PlayerCameraState, WorldCamera},
        shooting::components::PlayerWeapons,
    },
    world::world_objects::medkit::Medkit,
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

    let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft);
    let reloading = player_weapons.reloading;

    let speed = if shift_pressed && !reloading {
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

    let mut velocity = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        velocity += forward_camera * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        velocity -= right * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        velocity += right * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        velocity -= forward_camera * speed;
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_action_writer.write(MovementAction {
            direction: MovementDirection::Jump,
            character_controller_entity: player_entity,
        });
    }

    if velocity == Vec3::ZERO {
        return;
    }

    movement_action_writer.write(MovementAction {
        direction: MovementDirection::Move(velocity),
        character_controller_entity: player_entity,
    });
}

const MAX_DISTANCE_SHAPE_CAST_MOVEMENT: f32 = 0.3;
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
            MovementDirection::Move(world_velocity) => {
                // exclude medkits because we want to be able to walk through medkits
                let excluded_entities: Vec<Entity> = medkit_query
                    .iter()
                    .chain(std::iter::once(character_controller_entity))
                    .collect();

                let spatial_query_filter = &SpatialQueryFilter::default()
                    .with_excluded_entities(excluded_entities.clone());

                apply_collide_and_slide(
                    &mut velocity,
                    world_velocity,
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

fn apply_collide_and_slide(
    current_velocity: &mut Vec3,
    desired_velocity: Vec3,
    origin_transform: &Transform,
    spatial_query: &mut SpatialQuery,
    spatial_query_filter: &SpatialQueryFilter,
    time_delta_secs: f32,
    current_hit_count: usize,
) {
    const MAX_HITS: usize = 5;
    let Ok(direction_from_world_velocity) = Dir3::new(desired_velocity) else {
        return;
    };

    if desired_velocity.length_squared() < 0.0001 {
        *current_velocity = Vec3::splat(0.0);
        return;
    }

    if current_hit_count > MAX_HITS {
        *current_velocity = Vec3::splat(0.);
        return;
    }

    let ray_origin = origin_transform.translation
        - direction_from_world_velocity.as_vec3() * 0.025;

    if let Some(hit_ahead) = spatial_query.cast_shape(
        &Collider::capsule(CHARACTER_CAPSULE_RADIUS, CHARACTER_CAPSULE_LENGTH),
        ray_origin,
        origin_transform.rotation,
        direction_from_world_velocity,
        &ShapeCastConfig {
            max_distance: MAX_DISTANCE_SHAPE_CAST_MOVEMENT,
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
            *current_velocity = desired_velocity.reject_from_normalized(normal);

            // slope snapping
            let ray_down_origin = origin_transform.translation + Vec3::Y * 0.5;
            let ray_down_direction = Dir3::NEG_Y;
            let max_down_distance = 1.0;

            if let Some(hit_down) = spatial_query.cast_ray(
                ray_down_origin,
                ray_down_direction,
                max_down_distance,
                true,
                spatial_query_filter,
            ) {
                let hit_down_point =
                    ray_down_origin + ray_down_direction * hit_down.distance;
                let hit_down_y = hit_down_point.y;
                let player_y = origin_transform.translation.y;
                let difference_y = hit_down_y - player_y;
                if difference_y.abs() < 0.3 {
                    debug!("Snapping character controller to slope");
                    current_velocity.y = difference_y / time_delta_secs;
                }
            }
        } else {
            debug!("MOVEMENT: Obstacle in the way, sliding along wall");
            // not climable, e.g. a wall. we want to slide along the wall,
            // similar to the collide and slide algorithm
            // the main difference is that we ignore the Y part,
            // because its too step, so we dont want to climb up
            let impulse = desired_velocity.reject_from_normalized(normal);
            // we need to check again if the new velocity (impulse) would also penetrate an
            // obstacle until we dont or we reach MAX_HITS, where we just zero out velocity

            // update our transform so shape cast origin is correct
            let new_transform = Transform {
                translation: origin_transform.translation
                    + desired_velocity * time_delta_secs,
                rotation: origin_transform.rotation,
                scale: origin_transform.scale,
            };

            apply_collide_and_slide(
                current_velocity,
                impulse,
                &new_transform,
                spatial_query,
                spatial_query_filter,
                time_delta_secs,
                current_hit_count + 1,
            );
        }
    } else {
        debug!("MOVEMENT: No obstacle ahead, free movement");
        // no obstacle ahead, free movement
        current_velocity.x = desired_velocity.x;
        current_velocity.z = desired_velocity.z;
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

// Apply damping in the XZ Plane, basically this is deceleration over time
pub fn apply_movement_damping(
    query: Query<&mut LinearVelocity, With<CharacterController>>,
) {
    for mut velocity in query {
        velocity.x *= 0.9;
        velocity.z *= 0.9;
    }
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
