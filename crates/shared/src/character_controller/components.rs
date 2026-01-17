use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A marker component indicating that an entity is using a character controller
#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct CharacterController;

#[derive(
    Component, Default, Serialize, Deserialize, PartialEq, Clone, Debug,
)]
pub struct Grounded(pub bool);
