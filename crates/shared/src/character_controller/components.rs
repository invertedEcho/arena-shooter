use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A marker component indicating that an entity is using a character controller
#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct CharacterController;

#[derive(
    Component, Default, Serialize, Deserialize, PartialEq, Clone, Debug,
)]
pub struct Grounded(pub bool);

/// A marker component indicating that an entity is using a kinematic rigid body, and thus needs
/// to be controlled via the game itself. For example, gravity system runs on all entities with
/// this component.
#[derive(Component)]
pub struct KinematicEntity;
