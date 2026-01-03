use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    GRAVITY,
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
        GROUND_CASTER_MAX_DISTANCE, JUMP_VELOCITY, MAX_SLOPE_ANGLE,
        components::{CharacterController, Grounded},
    },
    protocol::{Inputs, Movement},
};

pub fn shared_movement(
    velocity: &mut LinearVelocity,
    input: &Inputs,
    spatial_query: &mut SpatialQuery,
    transform: &Transform,
    excluded_entities: Vec<Entity>,
) {
    match input {
        Inputs::Jump => {
            velocity.y = JUMP_VELOCITY;
            // if grounded.0 {
            //     velocity.y = JUMP_VELOCITY;
            // }
        }
        Inputs::Movement(movement) => {
            // exclude medkits because we want to be able to walk through medkits
            // let excluded_entities: Vec<Entity> = medkit_query
            //     .iter()
            //     .chain(std::iter::once(character_controller_entity))
            //     .collect();

            // let spatial_query_filter = &SpatialQueryFilter::default()
            //     .with_excluded_entities(excluded_entities.clone());

            // origin entity is player, this needs to exist as on server, the shape cast will hit
            // the player. but why not on the client btw?
            let spatial_query_filter = &SpatialQueryFilter::default()
                .with_excluded_entities(excluded_entities);
            let desired_velocity =
                convert_movement_to_desired_velocity(movement);

            apply_collide_and_slide(
                velocity,
                desired_velocity,
                transform,
                spatial_query,
                spatial_query_filter,
                1.0 / 60.0,
                0,
            );
        }
    }
}

fn convert_movement_to_desired_velocity(movement: &Movement) -> Vec3 {
    let mut desired_velocity: Vec3 = vec3(0.0, 0.0, 0.0);
    if movement.backwards {
        desired_velocity.z += 1.0;
    }
    if movement.forward {
        desired_velocity.z -= 1.0;
    }
    if movement.left {
        desired_velocity.x -= 1.0;
    }
    if movement.right {
        desired_velocity.x += 1.0;
    }

    desired_velocity
}

const MAX_DISTANCE_SHAPE_CAST_MOVEMENT: f32 = 0.3;

fn apply_collide_and_slide(
    current_velocity: &mut Vec3,
    desired_velocity: Vec3,
    origin_transform: &Transform,
    spatial_query: &mut SpatialQuery,
    spatial_query_filter: &SpatialQueryFilter,
    fixed_dt: f32,
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
        info!(
            "Got shape cast hit, entity that was hit: {}",
            hit_ahead.entity
        );
        // obstacle in the way, check if we can slimb it
        // a normal is just a direction something is facing
        let normal = hit_ahead.normal1;
        let slope_angle = normal.angle_between(Vec3::Y);
        let slope_climable = slope_angle < MAX_SLOPE_ANGLE;

        if slope_climable {
            info!("MOVEMENT: Climable slope!");
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
                    info!("Snapping character controller to slope");
                    current_velocity.y = difference_y / fixed_dt;
                }
            }
        } else {
            info!("MOVEMENT: Obstacle in the way, sliding along wall");
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
                    + desired_velocity * fixed_dt,
                rotation: origin_transform.rotation,
                scale: origin_transform.scale,
            };

            apply_collide_and_slide(
                current_velocity,
                impulse,
                &new_transform,
                spatial_query,
                spatial_query_filter,
                fixed_dt,
                current_hit_count + 1,
            );
        }
    } else {
        info!("MOVEMENT: No obstacle ahead, free movement");
        // no obstacle ahead, free movement
        current_velocity.x = desired_velocity.x;
        current_velocity.z = desired_velocity.z;
    }
}

/// Updates the [`Grounded`] status for character controllers.
pub fn update_on_ground(
    mut query: Query<
        (Entity, &mut Grounded, &mut LinearVelocity, &Transform),
        With<CharacterController>,
    >,
    spatial_query: SpatialQuery,
) {
    for (character_controller_entity, mut grounded, mut velocity, transform) in
        &mut query
    {
        let Some(_) = spatial_query.cast_shape(
            &Collider::capsule(
                CHARACTER_CAPSULE_RADIUS,
                CHARACTER_CAPSULE_LENGTH,
            ),
            transform.translation,
            transform.rotation,
            Dir3::NEG_Y,
            &ShapeCastConfig {
                max_distance: GROUND_CASTER_MAX_DISTANCE,
                ..default()
            },
            &SpatialQueryFilter::default()
                .with_excluded_entities([character_controller_entity]),
        ) else {
            info!("not grounded");
            grounded.0 = false;
            continue;
        };
        info!("We are grounded!");

        if !grounded.0 {
            grounded.0 = true;
        }

        if grounded.0 && velocity.y < 0.0 {
            velocity.y = 0.0
        }
    }
}

pub fn apply_gravity_over_time(
    query: Query<(&Grounded, &mut LinearVelocity), With<CharacterController>>,
) {
    for (grounded, mut velocity) in query {
        if !grounded.0 {
            velocity.y -= GRAVITY * 1.0 / 60.0;
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
        velocity.y -= 0.1;
    }
}
