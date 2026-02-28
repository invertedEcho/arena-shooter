use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// info about the world object collectible. this gets synced to all clients.
/// if active == false, then this world object collectible is on cooldown,
/// and clients will react to this change and hide the 3d model
#[derive(Component, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorldObjectCollectibleServerSide {
    pub active: bool,
    pub object_type: WorldObjectCollectibleType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum WorldObjectCollectibleType {
    Medkit,
    Ammunition,
}
