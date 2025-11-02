use bevy::prelude::*;

/// * 0: The player entity in which the cameras should be inserted into
#[derive(Message)]
pub struct SpawnPlayerCamerasMessage(pub Entity);
