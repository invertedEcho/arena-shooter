use avian3d::prelude::*;
use bevy::prelude::*;

pub mod components;

pub const CHARACTER_CAPSULE_RADIUS: f32 = 0.2;
pub const CHARACTER_CAPSULE_LENGTH: f32 = 1.3;

pub const MAX_DISTANCE_SHAPE_CAST_CHARACTER_CONTROLLER: f32 = 0.3;

pub const CHARACTER_HEIGHT: f32 =
    CHARACTER_CAPSULE_LENGTH + CHARACTER_CAPSULE_RADIUS * 2.0;

pub const LOCAL_FEET_CHARACTER: f32 = -1.0;

pub const MAX_SLOPE_ANGLE: f32 = 45.0_f32.to_radians();

pub const WALK_VELOCITY: f32 = 1.5;
pub const RUN_VELOCITY: f32 = 3.0;
pub const JUMP_VELOCITY: f32 = 3.0;

pub fn apply_collide_and_slide(
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
            max_distance: MAX_DISTANCE_SHAPE_CAST_CHARACTER_CONTROLLER,
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
    }
}
