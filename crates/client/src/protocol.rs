use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct PlayerPosition(pub Vec3);

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct PlayerId(pub u32);
