use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::GameStateServer;

#[derive(Serialize, Deserialize)]
pub struct ClientUpdatePositionMessage {
    pub new_translation: Vec3,
}

#[derive(Serialize, Deserialize)]
pub struct ShootRequest {
    pub origin: Vec3,
    pub direction: Dir3,
    // pub client_tick: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ChangeGameServerStateRequest(pub GameStateServer);

/// A client can send this message to the server indicating that the player requested a respawn.
/// The server will then update the players health and the players position.
#[derive(Serialize, Deserialize)]
pub struct ClientRespawnRequest;

/// This message is sent from server to client, so the client can spawn the damage indicator
#[derive(Serialize, Deserialize, Message, Copy, Clone)]
pub struct PlayerHitMessage {
    pub origin: Vec3,
}

/// The server will send this message to the client that the respawn was made and the client can now
/// update internal state, such as `InGameState`.
#[derive(Serialize, Deserialize)]
pub struct ConfirmRespawn;

/// This message gets sent to the client that the server detected is colliding with an ammunition
/// box. `PlayerWeapons` component currenctly lives on the client, so we can't directly increase
/// ammunition count. I like this approach (message) more, but maybe in the future we will have to
/// keep the `PlayerWeapons` component on the server anyways.
#[derive(Serialize, Deserialize)]
pub struct AmmunitionBoxCollected {
    pub ammunition_to_give: u64,
}
