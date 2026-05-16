use bevy::prelude::*;
use bincode::{Decode, Encode};

/// info about the world object collectible. this gets synced to all clients.
/// if active == false, then this world object collectible is on cooldown,
/// and clients will react to this change and hide the 3d model
#[derive(Component, Debug, Encode, Decode, PartialEq)]
pub struct WorldObjectCollectibleServerSide {
    pub active: bool,
    pub kind: WorldObjectCollectibleKind,
    pub position: Vec3,
}

#[derive(Debug, PartialEq, Copy, Clone, Encode, Decode)]
pub enum WorldObjectCollectibleKind {
    Medkit,
    Ammunition,
}
