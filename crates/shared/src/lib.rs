use avian3d::{PhysicsPlugins, prelude::*};
use bevy::prelude::*;
use std::net::{IpAddr, Ipv6Addr, SocketAddr, ToSocketAddrs};

use crate::{components::DespawnTimer, protocol::ProtocolPlugin};

pub mod character_controller;
pub mod collider_rules;
pub mod components;
pub mod messages;
pub mod player;
pub mod protocol;
pub mod utils;

pub const SERVER_PORT: u16 = 5888;
pub const AUTH_BACKEND_PORT: u16 = 4000;

pub const SERVER_ADDRESS_SERVER_SIDE: IpAddr =
    IpAddr::V6(Ipv6Addr::UNSPECIFIED);

pub fn get_server_socket_addr_client_side() -> SocketAddr {
    "game.invertedecho.com:5888"
        .to_socket_addrs()
        .expect("DNS resolution failed")
        .next()
        .unwrap()
}
pub fn get_auth_backend_socket_addr_client_side() -> SocketAddr {
    "game.invertedecho.com:4000"
        .to_socket_addrs()
        .expect("DNS resolution failed")
        .next()
        .unwrap()
}

pub const SERVER_SOCKET_ADDR_SERVER_SIDE: SocketAddr =
    SocketAddr::new(SERVER_ADDRESS_SERVER_SIDE, SERVER_PORT);

pub const AUTH_BACKEND_ADDRESS_SERVER_SIDE: SocketAddr =
    SocketAddr::new(SERVER_ADDRESS_SERVER_SIDE, AUTH_BACKEND_PORT);

pub const NETCODE_PROTOCOL_VERSION: u64 = 0;

pub const GRAVITY: f32 = 9.81;

pub const TINY_TOWN_MAP_PATH: &str = "maps/tiny_town/main.gltf";
pub const MEDIUM_PLASTIC_MAP_PATH: &str = "maps/medium_plastic/scene.gltf";
pub const SPAWN_POINT_MEDIUM_PLASTIC_MAP: Vec3 = vec3(0.0, 15.0, 0.0);

/// Functionality that runs on both client and server
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);

        app.add_plugins(PhysicsPlugins::default().build())
            // .add_plugins(PhysicsDebugPlugin)
            .insert_resource(Gravity(Vec3::NEG_Y * GRAVITY));
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
