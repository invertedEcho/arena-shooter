use avian3d::{
    PhysicsPlugins,
    prelude::{
        Gravity, IslandPlugin, IslandSleepingPlugin, PhysicsDebugPlugin,
        PhysicsInterpolationPlugin, PhysicsTransformPlugin,
    },
};
use bevy::prelude::*;
use lightyear::{
    avian3d::plugin::{AvianReplicationMode, LightyearAvianPlugin},
    input::native::plugin::InputPlugin,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::protocol::ProtocolPlugin;

pub mod collider_rules;
pub mod components;
pub mod messages;
pub mod player;
pub mod protocol;

const SERVER_PORT: u16 = 5888;
pub const SERVER_ADDRESS: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);

pub const GRAVITY: f32 = 9.81;

pub const TINY_TOWN_MAP_PATH: &str = "maps/tiny_town/main.gltf";
pub const MEDIUM_PLASTIC_MAP_PATH: &str = "maps/medium_plastic/scene.gltf";

/// Functionality that runs on both client and server
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);

        app.add_plugins(LightyearAvianPlugin {
            replication_mode: AvianReplicationMode::Position,
            ..default()
        });

        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                // these interfere with lightyear avian plugins
                .disable::<PhysicsTransformPlugin>()
                .disable::<PhysicsInterpolationPlugin>()
                .disable::<IslandPlugin>()
                .disable::<IslandSleepingPlugin>(),
        )
        .add_plugins(PhysicsDebugPlugin)
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
