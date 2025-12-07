use bevy::prelude::*;

pub fn is_facing_target(origin: &Transform, target: &Transform) -> bool {
    // 3 degrees
    const TOLERANCE_RADIUS: f32 = 3.0_f32.to_radians();
    // Direction origin is currently facing
    let forward = origin.forward();

    // Direction toward the target
    let mut to_target = target.translation - origin.translation;
    to_target = to_target.normalize();

    // Angle between them
    let angle = forward.angle_between(to_target);

    angle < TOLERANCE_RADIUS
}
