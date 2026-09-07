use bevy::prelude::*;
use netvy::prelude::*;

use crate::{
    components::Health,
    enemy::components::{Enemy, EnemyState},
    game_score::GameScore,
    multiplayer_messages::{
        ClientCommand, ClientRespawnRequest, ConfirmRespawn, PlayerHitMessage,
        ShootRequest,
    },
    player::{Player, PlayerState},
    shooting::{PlayerKilled, PlayerWeapons},
    world_object::WorldObjectCollectibleServerSide,
};

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_network_message::<ShootRequest>(
            MessageDirection::ClientToServer,
        );

        app.register_network_message::<ClientRespawnRequest>(
            MessageDirection::ClientToServer,
        );

        app.register_network_message::<ConfirmRespawn>(
            MessageDirection::ServerToClient,
        );

        app.register_network_message::<ClientCommand>(
            MessageDirection::ClientToServer,
        );

        app.register_network_message::<PlayerHitMessage>(
            MessageDirection::ServerToClient,
        );

        app.register_network_message::<ClientCommand>(
            MessageDirection::ClientToServer,
        );

        app.register_network_message::<PlayerKilled>(
            MessageDirection::ServerToClients,
        );

        app.register_component::<Player>();
        app.register_component::<PlayerState>();
        app.register_component::<PlayerWeapons>();
        app.register_component::<PlayerWeapons>();

        app.register_component::<Health>();

        app.register_component::<Enemy>();
        app.register_component::<EnemyState>();

        app.register_component::<GameScore>();

        app.register_component::<WorldObjectCollectibleServerSide>();
    }
}
