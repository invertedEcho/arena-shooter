use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::input::native::InputMarker;
use lightyear::prelude::*;
use shared::SERVER_ADDRESS;
use shared::player::Player;
use shared::protocol::{ClientUpdatePositionMessage, PositionUpdateChannel};

use crate::ClientId;
use crate::character_controller::components::CharacterControllerBundle;
use crate::character_controller::{
    CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
};
use crate::game_flow::states::AppState;
use crate::player::camera::messages::SpawnPlayerCamerasMessage;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ConnectToServerMessage>();
        app.add_systems(
            Update,
            (
                handle_connect_to_server_message,
                send_client_update_position,
            ),
        );
        app.add_observer(handle_new_player);
    }
}

#[derive(Message)]
pub struct ConnectToServerMessage;

pub fn handle_connect_to_server_message(
    mut commands: Commands,
    mut message_reader: MessageReader<ConnectToServerMessage>,
    client_id: Res<ClientId>,
) {
    for _ in message_reader.read() {
        info!("connecting to server...");

        let auth = Authentication::Manual {
            server_addr: SERVER_ADDRESS,
            client_id: client_id.0,
            private_key: Key::default(),
            protocol_id: 0,
        };

        let client = commands
            .spawn((
                Name::new("Client"),
                Client::default(),
                LocalAddr("127.0.0.1:0".parse().unwrap()),
                PeerAddr(SERVER_ADDRESS),
                Link::new(None),
                ReplicationReceiver::default(),
                NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
                UdpIo::default(),
            ))
            .id();

        // Send connect request
        commands.trigger(Connect { entity: client });
    }
}

/// This resource exists so we can easily retrieve the player entity throughout the game
#[derive(Resource)]
pub struct ClientLocalPlayer(pub Entity);

fn handle_new_player(
    trigger: On<Add, Player>,
    mut commands: Commands,
    player_query: Query<Has<Controlled>, With<Player>>,
    mut spawn_player_camera_message_writer: MessageWriter<
        SpawnPlayerCamerasMessage,
    >,
) {
    if let Ok(is_controlled) = player_query.get(trigger.entity)
        && is_controlled
    {
        info!(
            "We found our player, storing it in ClientLocalPlayer resource {}",
            trigger.entity
        );
        commands.insert_resource(ClientLocalPlayer(trigger.entity));

        // we insert the character controller locally on our client, as it should only run on the
        // client. as it is not registered in our protocol, it wont be replicated.
        commands.entity(trigger.entity).insert((
            Replicate::to_server(),
            CharacterControllerBundle::default(),
            DespawnOnExit(AppState::InGame),
        ));
        spawn_player_camera_message_writer
            .write(SpawnPlayerCamerasMessage(trigger.entity));
    } else {
        info!(
            "A player was added, but it doesn't have Controlled Component, \
             e.g. its not our player!"
        );
    }
}

pub fn send_client_update_position(
    // With<Client> also ensures its the messagesender from our local client, as client component
    // only gets inserted into our own client
    mut message_sender: Single<
        &mut MessageSender<ClientUpdatePositionMessage>,
        With<Client>,
    >,
    // TODO: only get our own Player
    player_transform: Single<&Transform, (With<Player>)>,
) {
    message_sender.send::<PositionUpdateChannel>(ClientUpdatePositionMessage {
        new_transform: **player_transform,
    });
}
