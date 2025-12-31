use bevy::prelude::*;
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use shared::SERVER_ADDRESS;

use crate::ClientId;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<StartMultiplayerMessage>();
        app.add_systems(Update, handle_start_multiplayer_message);
    }
}

#[derive(Message)]
pub struct StartMultiplayerMessage;

pub fn handle_start_multiplayer_message(
    mut commands: Commands,
    mut message_reader: MessageReader<StartMultiplayerMessage>,
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
