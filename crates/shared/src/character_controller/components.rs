use avian3d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::character_controller::{
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
            // FIXME: should probably start with false
            grounded: Grounded(true),
        }
    }
}

#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct Grounded(pub bool);

impl Default for Grounded {
    fn default() -> Self {
        Self(true)
    }
}
