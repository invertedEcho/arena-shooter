use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// info about the world object collectible. this gets synced to all clients.
/// if active == false, then this world object collectible is on cooldown,
/// and clients will react to this change and hide the 3d model
#[derive(Component, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorldObjectCollectibleServerSide {
    pub active: bool,
    pub kind: WorldObjectCollectibleKind,
    pub position: Vec3,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum WorldObjectCollectibleKind {
    Medkit,
    Ammunition,
}
