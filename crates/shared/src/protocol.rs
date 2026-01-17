use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    components::Health,
    enemy::components::{Enemy, EnemyState},
    player::Player,
};

pub struct OrderedReliableMessageChannel;
pub struct SequencedUnreliableChannel;

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
    pub origin: Vec3,
    pub direction: Dir3,
    // pub client_tick: u32,
}

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_channel::<SequencedUnreliableChannel>(ChannelSettings {
            mode: ChannelMode::SequencedUnreliable,
            ..default()
        })
        .add_direction(NetworkDirection::ClientToServer);

        app.add_channel::<OrderedReliableMessageChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<ClientUpdatePositionMessage>()
            .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<ShootRequest>()
            .add_direction(NetworkDirection::ClientToServer);

        app.register_component::<Player>();

        app.register_component::<PlayerPositionServer>();

        app.register_component::<Health>();

        app.register_component::<Enemy>();
        app.register_component::<EnemyState>();
    }
}
