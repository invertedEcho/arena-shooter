use bevy::prelude::*;

#[derive(Message)]
pub struct SpawnDebugPointMessage(pub Transform);
