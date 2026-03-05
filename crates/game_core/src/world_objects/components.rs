use bevy::prelude::*;

#[derive(Component)]
pub struct RespawnTimer(pub Timer);

// we need this marker component so we can despawn the map model when `StopGame` message is
// written.
#[derive(Component)]
pub struct MapModel;
