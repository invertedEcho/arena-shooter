use bevy::prelude::*;

pub fn is_facing_target_without_y(
    origin: &Transform,
    target: &Transform,
) -> bool {
    // 3 degrees
    const TOLERANCE_RADIUS: f32 = 3.0_f32.to_radians();
    // Direction origin is currently facing
    let mut forward = origin.forward().as_vec3();

    // Direction toward the target
    let mut to_target = target.translation - origin.translation;
    to_target = to_target.normalize();

    // ignore y because we dont care about it
    forward.y = 0.0;
    to_target.y = 0.0;

    // Angle between them
    let angle = forward.angle_between(to_target);

    angle < TOLERANCE_RADIUS
}
