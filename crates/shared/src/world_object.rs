use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// info about the world object collectible. this gets synced to all clients.
/// if active == false, then this world object collectible is on cooldown,
/// and clients will react to this change and hide the 3d model
#[derive(Component, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldObjectCollectibleServerSide {
    pub active: bool,
    pub kind: WorldObjectCollectibleKind,
    pub position: Vec3,
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum WorldObjectCollectibleKind {
    Medkit,
    Ammunition,
}
