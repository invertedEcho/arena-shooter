use avian3d::{math::Quaternion, prelude::*};
use bevy::prelude::*;

use crate::character_controller::{
    CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
};

#[derive(Component, Debug, Reflect, PartialEq)]
pub enum MovementState {
    Idle,
    Walking,
    Running,
}

/// A marker component indicating that an entity is using a character controller
#[derive(Component)]
pub struct CharacterController;

#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    rigid_body: RigidBody,
    collider: Collider,
    locked_axes: LockedAxes,
    movement_state: MovementState,
    grounded: Grounded,
    ground_caster: ShapeCaster,
}

impl Default for CharacterControllerBundle {
    fn default() -> Self {
        Self {
            character_controller: CharacterController,
            rigid_body: RigidBody::Kinematic,
            collider: Collider::capsule(
                CHARACTER_CAPSULE_RADIUS,
                CHARACTER_CAPSULE_LENGTH,
            ),
            locked_axes: LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            movement_state: MovementState::Idle,
            grounded: Grounded(true),
            ground_caster: ShapeCaster::new(
                Collider::capsule(
                    CHARACTER_CAPSULE_RADIUS,
                    CHARACTER_CAPSULE_LENGTH,
                ),
                Vec3::ZERO,
                Quaternion::default(),
                Dir3::NEG_Y,
            )
            .with_max_distance(0.1),
        }
    }
}

#[derive(Component)]
pub struct Grounded(pub bool);

impl Default for Grounded {
    fn default() -> Self {
        Self(true)
    }
}
