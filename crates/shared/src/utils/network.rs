use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr, ToSocketAddrs},
    thread::sleep,
    time::Duration,
};

use bevy::prelude::*;

pub const SERVER_SOCKET_ADDR_SINGLEPLAYER: SocketAddr =
    SocketAddr::new(SERVER_ADDRESS_SERVER_SIDE, 0);

pub const SERVER_SOCKET_ADDR_DEDICATED_SERVER: SocketAddr =
    SocketAddr::new(SERVER_ADDRESS_SERVER_SIDE, SERVER_PORT);

pub const AUTH_BACKEND_ADDRESS_SERVER_SIDE: SocketAddr =
    SocketAddr::new(SERVER_ADDRESS_SERVER_SIDE, AUTH_BACKEND_PORT);

pub const NETCODE_PROTOCOL_VERSION: u64 = 0;

pub const SERVER_PORT: u16 = 5888;
pub const AUTH_BACKEND_PORT: u16 = 4000;

pub const SERVER_ADDRESS_SERVER_SIDE: IpAddr =
    IpAddr::V6(Ipv6Addr::UNSPECIFIED);

fn resolve_with_retry(
    address: &str,
) -> std::io::Result<Vec<std::net::SocketAddr>> {
    const RETRY_COUNT: u8 = 3;
    let mut last_err = None;

    for _ in 0..RETRY_COUNT {
        match address.to_socket_addrs() {
            Ok(addrs) => return Ok(addrs.collect()),
            Err(error) => {
                eprintln!(
                    "Failed to resolve game server dns. Retrying, sleeping \
                     for 100ms"
                );
                last_err = Some(error);
                sleep(Duration::from_millis(100));
            }
        }
    }

    Err(last_err.unwrap())
}

pub fn get_dedicated_server_socket_addr_client_side() -> Option<SocketAddr> {
    match resolve_with_retry("game.invertedecho.com:5888") {
        Ok(success) => success.first().copied(),
        Err(error) => {
            warn!("Failed to resolve game server: {}", error);
            None
        }
    }
}
pub fn get_auth_backend_socket_addr_client_side() -> Option<SocketAddr> {
    match resolve_with_retry("game.invertedecho.com:4000") {
        Ok(success) => success.first().copied(),
        Err(error) => {
            warn!("Failed to resolve game auth server: {}", error);
            None
        }
    }
}
