use bevy::prelude::*;
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::input::native::InputMarker;
use lightyear::prelude::*;
use shared::SERVER_ADDRESS;
use shared::player::Player;
use shared::protocol::Inputs;

use crate::ClientId;
use crate::player::camera::messages::SpawnPlayerCamerasMessage;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ConnectToServerMessage>();
        app.add_systems(Update, handle_connect_to_server_message);
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
    trigger: On<Add, (Player, Predicted)>,
    mut commands: Commands,
    player_query: Query<Has<Controlled>, (With<Predicted>, With<Player>)>,
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
        commands
            .entity(trigger.entity)
            .insert(InputMarker::<Inputs>::default());
        spawn_player_camera_message_writer
            .write(SpawnPlayerCamerasMessage(trigger.entity));
    }
}
