use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, PartialEq, Serialize, Deserialize)]
pub struct Health(pub f32);

/// Insert this component into entities that you want to have despawned when the given Timer has
/// reached its end. The timer will be automatically ticked in `src/common/systems.rs`
#[derive(Component)]
pub struct DespawnTimer(pub Timer);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MedkitSpawnLocation;
