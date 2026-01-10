use avian3d::{PhysicsPlugins, prelude::*};
use bevy::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

use crate::protocol::ProtocolPlugin;

pub mod collider_rules;
pub mod components;
pub mod messages;
pub mod player;
pub mod protocol;
pub mod utils;

const SERVER_PORT: u16 = 5888;
pub const SERVER_ADDRESS: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), SERVER_PORT);
pub const CLIENT_PORT: u16 = 0;

pub const AUTH_BACKEND_PORT: u16 = 4000;

pub const AUTH_BACKEND_ADDRESS: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, AUTH_BACKEND_PORT));

pub const GRAVITY: f32 = 9.81;

pub const NETCODE_PROTOCOL_VERSION: u64 = 0;

pub const TINY_TOWN_MAP_PATH: &str = "maps/tiny_town/main.gltf";
pub const MEDIUM_PLASTIC_MAP_PATH: &str = "maps/medium_plastic/scene.gltf";
pub const SPAWN_POINT_MEDIUM_PLASTIC_MAP: Vec3 = vec3(0.0, 15.0, 0.0);

pub const CHARACTER_CAPSULE_RADIUS: f32 = 0.2;
pub const CHARACTER_CAPSULE_LENGTH: f32 = 1.3;

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
