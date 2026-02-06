pub mod components;
pub mod messages;

pub const CHARACTER_CAPSULE_RADIUS: f32 = 0.2;
pub const CHARACTER_CAPSULE_LENGTH: f32 = 1.3;

pub const CHARACTER_HEIGHT: f32 =
    CHARACTER_CAPSULE_LENGTH + CHARACTER_CAPSULE_RADIUS * 2.0;

pub const LOCAL_FEET_CHARACTER: f32 = -1.0;

pub const MAX_SLOPE_ANGLE: f32 = 45.0_f32.to_radians();

pub const WALK_VELOCITY: f32 = 1.5;
pub const RUN_VELOCITY: f32 = 3.0;
pub const JUMP_VELOCITY: f32 = 3.0;
