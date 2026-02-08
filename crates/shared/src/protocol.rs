use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    GameModeServer,
    components::Health,
    enemy::components::{Enemy, EnemyState},
    game_score::GameScore,
    player::Player,
};

pub struct OrderedReliableMessageChannel;
pub struct SequencedUnreliableChannel;

#[derive(Serialize, Deserialize)]
pub struct ClientUpdatePositionMessage {
    pub new_translation: Vec3,
}

/// This component indicates the current location of an entity on the server. It is replicated to
/// all clients. All clients interpolate the local transform of this entity to this component
#[derive(Component, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EntityPositionServer {
    pub translation: Vec3,
}

#[derive(Serialize, Deserialize)]
pub struct ShootRequest {
    pub origin: Vec3,
    pub direction: Dir3,
    /// Whether this ShootRequest is coming from an enemy. We need this as enemies dont have
    /// "ControlledBy" component, and as such need different handling in handle_shoot_requests
    pub from_enemy: bool,
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

        app.register_component::<EntityPositionServer>();

        app.register_component::<Health>();

        app.register_component::<Enemy>();
        app.register_component::<EnemyState>();

        app.register_component::<GameModeServer>();

        app.register_component::<GameScore>();

        // FIXME: medkit should be spawned on server, replicated to clients, and only clients
        // visually rotate them
        // app.register_component::<Medkit>();
    }
}
