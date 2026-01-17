use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use async_compat::Compat;
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use lightyear::netcode::ConnectToken;
use lightyear::utils::collections::HashSet;
use shared::NETCODE_PROTOCOL_VERSION;
use tokio::io::AsyncWriteExt;

/// This resource will track the list of Netcode client-ids currently in use, so that
/// we don't have multiple clients with the same id
#[derive(Resource, Default)]
pub struct ClientIds(pub Arc<RwLock<HashSet<u64>>>);

/// Start a detached task that listens for incoming TCP connections and sends `ConnectToken`s to clients
pub fn start_netcode_authentication_task(
    game_server_addr: SocketAddr,
    auth_backend_addr: SocketAddr,
    client_ids: Arc<RwLock<HashSet<u64>>>,
    server_private_key: [u8; 32],
) {
    IoTaskPool::get()
        .spawn(Compat::new(async move {
            info!(
                "Listening for ConnectToken requests on {}",
                auth_backend_addr
            );
            let listener = tokio::net::TcpListener::bind(auth_backend_addr)
                .await
                .unwrap();
            loop {
                // received a new connection
                let (mut stream, _) = listener.accept().await.unwrap();

                // assign a new client_id
                let client_id = loop {
                    let client_id = rand::random();
                    if !client_ids.read().unwrap().contains(&client_id) {
                        break client_id;
                    }
                };

                let token = ConnectToken::build(
                    game_server_addr,
                    NETCODE_PROTOCOL_VERSION,
                    client_id,
                    server_private_key,
                )
                .generate()
                .expect("Failed to generate token");

                let serialized_token =
                    token.try_into_bytes().expect("Failed to serialize token");
                trace!(
                    "Sending token {:?} to client {}. Token len: {}",
                    serialized_token,
                    client_id,
                    serialized_token.len()
                );
                stream
                    .write_all(&serialized_token)
                    .await
                    .expect("Failed to send token to client");
            }
        }))
        .detach();
}
