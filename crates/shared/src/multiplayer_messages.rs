use bevy::prelude::*;
use netvy::PeerId;
use serde::{Deserialize, Serialize};

use crate::{GameMap, GameMode, GameStateServer};

#[derive(Message, Serialize, Deserialize)]
pub struct ShootRequest {
    pub origin: Vec3,
    pub direction: Dir3,
    pub source_peer_id: PeerId,
}

/// A client can send this to the server to request update of the game config on the server, such as
/// the current map or game mode.
#[derive(Serialize, Deserialize, Debug)]
pub enum ClientCommand {
    SetGameMode(GameMode),
    SetMap(GameMap),
    SetState(GameStateServer),
}

/// A client can send this message to the server indicating that the player requested a respawn.
/// The server will then update the players health and the players position.
#[derive(Serialize, Deserialize)]
pub struct ClientRespawnRequest;

/// This message is sent from server to client, whenever another player/enemy shot the player of
/// that client
#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct PlayerHitMessage {
    pub origin: Vec3,
}

/// The server will send this message to the client that the respawn was made and the client can now
/// update internal state, such as `InGameState`.
#[derive(Serialize, Deserialize)]
pub struct ConfirmRespawn;
