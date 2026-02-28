use bevy::prelude::*;
use lightyear::prelude::*;

use crate::{
    GameModeServer,
    components::{EntityPositionServer, Health},
    enemy::components::{Enemy, EnemyState},
    game_score::GameScore,
    multiplayer_messages::{
        ChangeGameServerStateRequest, ClientRespawnRequest,
        ClientUpdatePositionMessage, ConfirmRespawn, PlayerHitMessage,
        ShootRequest,
    },
    player::{Player, PlayerState},
    world_object::WorldObjectCollectibleServerSide,
};

pub struct OrderedReliableChannel;
pub struct SequencedUnreliableChannel;

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_channel::<SequencedUnreliableChannel>(ChannelSettings {
            mode: ChannelMode::SequencedUnreliable,
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);

        app.add_channel::<OrderedReliableChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);

        app.register_message::<ClientUpdatePositionMessage>()
            .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<ShootRequest>()
            .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<ClientRespawnRequest>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<ConfirmRespawn>()
            .add_direction(NetworkDirection::ServerToClient);

        app.register_message::<ChangeGameServerStateRequest>()
            .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<PlayerHitMessage>()
            .add_direction(NetworkDirection::ServerToClient);

        app.register_component::<Player>();
        app.register_component::<PlayerState>();

        app.register_component::<EntityPositionServer>();

        app.register_component::<Health>();

        app.register_component::<Enemy>();
        app.register_component::<EnemyState>();

        app.register_component::<GameModeServer>();

        app.register_component::<GameScore>();

        app.register_component::<WorldObjectCollectibleServerSide>();
    }
}
