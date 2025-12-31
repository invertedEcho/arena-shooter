use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;

const SERVER_PORT: u16 = 5888;
pub const SERVER_ADDRESS: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);

pub fn setup_server(mut commands: Commands) {
    // Create server entity
    let server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig::default()),
            LocalAddr(SERVER_ADDRESS),
            ServerUdpIo::default(),
        ))
        .id();

    // Tell server to start listening
    commands.trigger(Start { entity: server });
}

pub fn handle_new_client(trigger: On<Add, Connected>) {
    info!("New client! {}", trigger.entity);
}
