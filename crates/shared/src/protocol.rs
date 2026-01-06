use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::player::{Health, Player};

pub struct PositionUpdateChannel;

#[derive(Serialize, Deserialize)]
pub struct ClientUpdatePositionMessage {
    pub new_translation: Vec3,
}

#[derive(Component, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlayerPositionServer {
    pub translation: Vec3,
}

#[derive(Serialize, Deserialize)]
pub struct ShootRequest {
    origin: Vec3,
    direction: Vec3,
    client_tick: u32,
}

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_channel::<PositionUpdateChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<ClientUpdatePositionMessage>()
            .add_direction(NetworkDirection::ClientToServer);

        app.register_component::<Player>();

        app.register_component::<PlayerPositionServer>();

        app.register_component::<Health>();
    }
}
