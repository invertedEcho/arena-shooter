use bevy::prelude::*;
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::*;

use crate::server::SERVER_ADDRESS;

pub fn setup_client(mut commands: Commands) {
    let auth = Authentication::Manual {
        server_addr: SERVER_ADDRESS,
        client_id: 0,
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
