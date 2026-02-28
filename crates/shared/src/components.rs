use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, PartialEq, Serialize, Deserialize, Reflect)]
pub struct Health(pub f32);

/// Insert this component into entities that you want to have despawned when the given Timer has
/// reached its end. The timer will be automatically ticked in `src/common/systems.rs`
#[derive(Component)]
pub struct DespawnTimer(pub Timer);

/// This component indicates the current location of an entity on the server. It is replicated to
/// all clients. All clients interpolate the local transform of this entity to this component
#[derive(Component, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EntityPositionServer {
    pub translation: Vec3,
}
