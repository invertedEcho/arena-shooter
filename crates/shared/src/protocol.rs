use bevy::prelude::*;
use netvy::prelude::*;

use crate::{
    components::Health,
    enemy::components::{Enemy, EnemyState},
    game_score::GameScore,
    multiplayer_messages::{
        ChangeGameServerStateRequest, ClientRespawnRequest, ConfirmRespawn,
        PlayerHitMessage, ShootRequest,
    },
    player::{Player, PlayerState},
    shooting::PlayerWeapons,
    world_object::WorldObjectCollectibleServerSide,
};

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // app.add_channel::<SequencedUnreliableChannel>(ChannelSettings {
        //     mode: ChannelMode::SequencedUnreliable,
        //     ..default()
        // })
        // .add_direction(NetworkDirection::Bidirectional);
        //
        // app.add_channel::<OrderedReliableChannel>(ChannelSettings {
        //     mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
        //     ..default()
        // })
        // .add_direction(NetworkDirection::Bidirectional);
        //
        app.register_net_message::<ShootRequest>(
            MessageDirection::ClientToServer,
        );

        app.register_net_message::<ClientRespawnRequest>(
            MessageDirection::ClientToServer,
        );

        app.register_net_message::<ConfirmRespawn>(
            MessageDirection::ServerToClient,
        );

        app.register_net_message::<ChangeGameServerStateRequest>(
            MessageDirection::ClientToServer,
        );

        app.register_net_message::<PlayerHitMessage>(
            MessageDirection::ServerToClient,
        );

        app.register_component::<Player>();
        app.register_component::<PlayerState>();
        app.register_component::<PlayerWeapons>();

        app.register_component::<Health>();

        app.register_component::<Enemy>();
        app.register_component::<EnemyState>();

        app.register_component::<GameScore>();

        app.register_component::<WorldObjectCollectibleServerSide>();
    }
}
