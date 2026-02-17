use avian3d::prelude::*;
use bevy::prelude::*;
use shared::character_controller::{
    CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
    components::{CharacterController, Grounded},
};

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
            grounded: Grounded(false),
        }
    }
}
