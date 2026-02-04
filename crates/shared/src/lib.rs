use avian3d::{PhysicsPlugins, prelude::*};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    env,
    net::{IpAddr, Ipv6Addr, SocketAddr, ToSocketAddrs},
    thread::sleep,
    time::Duration,
};

use crate::{
    character_controller::messages::MovementAction, components::DespawnTimer,
    protocol::ProtocolPlugin,
};

pub mod character_controller;
pub mod components;
pub mod enemy;
pub mod messages;
pub mod player;
pub mod protocol;
pub mod utils;

#[derive(Resource, PartialEq)]
pub enum ServerRunMode {
    Headless,
    Headful,
}

/// A resource existing to know whether we are using a remote server to connect to,
/// or a local server is started, e.g. for singleplayer
#[derive(States, PartialEq, Default, Debug, Hash, Clone, Eq)]
pub enum ServerMode {
    #[default]
    None,
    RemoteServer,
    LocalServerSinglePlayer,
}

/// The game mode that is running on the server. Must be a component as we can only replicate
/// components with lightyear.
#[derive(Component, Serialize, Deserialize, PartialEq, Debug)]
pub enum GameModeServer {
    Waves,
    FreeForAll,
    FreeRoam,
}

#[derive(Component)]
pub struct Medkit {
    pub active: bool,
    pub health_to_give: f32,
    pub float_direction: FloatDirection,
    pub respawn_timer: Timer,
}

pub enum FloatDirection {
    Up,
    Down,
}

pub fn load_private_key_from_env() -> Result<[u8; 32], String> {
    let Ok(value) = env::var("SERVER_PRIVATE_KEY") else {
        panic!("Please create a .env file containing a SERVER_PRIVATE_KEY.");
    };

    let bytes = hex::decode(&value)
        .map_err(|e| format!("Invalid hex in SERVER_PRIVATE_KEY: {e}"))?;

    if bytes.len() != 32 {
        return Err(format!(
            "SERVER_PRIVATE_KEY must be 32 bytes (got {})",
            bytes.len()
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

// Dependin whether this is for server binary or server locally for single player, it will either:
// - return private key from .env file (server binary)
// - return static private key, just zeroes (local server for singleplayer on the client)
pub fn get_private_key(server_mode: &ServerMode) -> [u8; 32] {
    const LOCAL_SERVER_PRIVATE_KEY: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];
    match server_mode {
        ServerMode::RemoteServer => load_private_key_from_env().unwrap(),
        ServerMode::LocalServerSinglePlayer => LOCAL_SERVER_PRIVATE_KEY,
        ServerMode::None => LOCAL_SERVER_PRIVATE_KEY,
    }
}

pub const SERVER_PORT: u16 = 5888;
pub const AUTH_BACKEND_PORT: u16 = 4000;

pub const SERVER_ADDRESS_SERVER_SIDE: IpAddr =
    IpAddr::V6(Ipv6Addr::UNSPECIFIED);

fn resolve_with_retry(
    address: &str,
) -> std::io::Result<Vec<std::net::SocketAddr>> {
    const RETRY_COUNT: u8 = 3;
    let mut last_err = None;

    for _ in 0..RETRY_COUNT {
        match address.to_socket_addrs() {
            Ok(addrs) => return Ok(addrs.collect()),
            Err(error) => {
                eprintln!(
                    "Failed to resolve game server dns. Retrying, sleeping \
                     for 100ms"
                );
                last_err = Some(error);
                sleep(Duration::from_millis(100));
            }
        }
    }

    Err(last_err.unwrap())
}

pub fn get_server_socket_addr_client_side() -> Option<SocketAddr> {
    match resolve_with_retry("game.invertedecho.com:5888") {
        Ok(success) => success.first().copied(),
        Err(error) => {
            warn!("Failed to resolve game server: {}", error);
            None
        }
    }
}
pub fn get_auth_backend_socket_addr_client_side() -> Option<SocketAddr> {
    match resolve_with_retry("game.invertedecho.com:4000") {
        Ok(success) => success.first().copied(),
        Err(error) => {
            warn!("Failed to resolve game auth server: {}", error);
            None
        }
    }
}

pub const SERVER_SOCKET_ADDR_SINGLEPLAYER: SocketAddr =
    SocketAddr::new(SERVER_ADDRESS_SERVER_SIDE, 0);

pub const AUTH_BACKEND_ADDRESS_SERVER_SIDE: SocketAddr =
    SocketAddr::new(SERVER_ADDRESS_SERVER_SIDE, AUTH_BACKEND_PORT);

pub const NETCODE_PROTOCOL_VERSION: u64 = 0;

pub const GRAVITY: f32 = 9.81;

pub const DEFAULT_BULLET_DAMAGE: f32 = 7.5;

pub const TINY_TOWN_MAP_PATH: &str = "maps/tiny_town/main.gltf";
pub const MEDIUM_PLASTIC_MAP_PATH: &str = "maps/medium_plastic/scene.gltf";
pub const SPAWN_POINT_MEDIUM_PLASTIC_MAP: Vec3 = vec3(0.0, 10.0, 0.0);

/// Functionality that runs on both client and server
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);

        app.add_message::<MovementAction>();

        app.add_plugins(PhysicsPlugins::default().build())
            // .add_plugins(PhysicsDebugPlugin)
            .insert_resource(Gravity(Vec3::NEG_Y * GRAVITY));
        app.add_systems(Update, handle_despawn_timer);
    }
}

// This is not a substate, as it needs to exist globally
#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
pub enum SelectedMapState {
    #[default]
    MediumPlastic,
    TinyTown,
}

pub fn handle_despawn_timer(
    despawn_timer_query: Query<(Entity, &mut DespawnTimer)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut timer) in despawn_timer_query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
