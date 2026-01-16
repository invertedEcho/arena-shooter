use avian3d::{math::Quaternion, prelude::*};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use shared::character_controller::{
    CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
};

/// A marker component indicating that an entity is using a character controller
#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct CharacterController;

#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    rigid_body: RigidBody,
    collider: Collider,
    locked_axes: LockedAxes,
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
            // TODO: should this tart with false or true? -> i think it doesnt really matter as it
            // will be updated by update_grounded system anyways
            grounded: Grounded(true),
            ground_caster: ShapeCaster::new(
                Collider::capsule(
                    CHARACTER_CAPSULE_RADIUS,
                    CHARACTER_CAPSULE_LENGTH,
                ),
                Vec3::ZERO,
                Quaternion::IDENTITY,
                Dir3::NEG_Y,
            )
            .with_max_distance(0.1),
        }
    }
}

#[derive(Component, Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Grounded(pub bool);

impl Default for Grounded {
    fn default() -> Self {
        Self(true)
    }
}
