use avian3d::{
    PhysicsPlugins,
    prelude::{
        Gravity, PhysicsDebugPlugin, PhysicsInterpolationPlugin,
        PhysicsTransformPlugin,
    },
};
use bevy::prelude::*;
use lightyear::avian3d::plugin::{AvianReplicationMode, LightyearAvianPlugin};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::{
    character_controller::CharacterControllerPlugin, protocol::ProtocolPlugin,
};

pub mod character_controller;
pub mod components;
pub mod messages;
pub mod player;
pub mod protocol;

const SERVER_PORT: u16 = 5888;
pub const SERVER_ADDRESS: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);

const GRAVITY: f32 = 9.81;

/// Functionality that runs on both client and server
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);

        // app.add_plugins(PredictionPlugin);
        app.add_plugins(LightyearAvianPlugin {
            replication_mode: AvianReplicationMode::Position,
            ..default()
        });

        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                // these interfere with lightyear avian plugins
                .disable::<PhysicsTransformPlugin>()
                .disable::<PhysicsInterpolationPlugin>(),
        )
        .add_plugins(PhysicsDebugPlugin)
        .insert_resource(Gravity(Vec3::NEG_Y * GRAVITY));

        // own plugins
        app.add_plugins(CharacterControllerPlugin);
    }
}
