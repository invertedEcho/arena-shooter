use bevy::prelude::*;

#[derive(Message)]
pub struct SpawnDebugPointMessage(pub Transform);

#[derive(Message)]
pub struct SpawnMapMessage;
