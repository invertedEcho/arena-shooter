use bevy::prelude::*;

#[derive(Message)]
pub struct SpawnDebugPointMessage(pub Vec3);

#[derive(Message)]
pub struct SpawnMapMessage;
