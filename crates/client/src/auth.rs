use std::net::SocketAddr;

use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{Task, block_on};
use lightyear::netcode::{CONNECT_TOKEN_BYTES, ConnectToken};
use lightyear::prelude::client::*;
use lightyear::prelude::*;

/// Holds a handle to an io task that is requesting a `ConnectToken` from the backend
#[derive(Resource)]
pub struct ConnectTokenRequestTask {
    pub auth_backend_addr: SocketAddr,
    pub task: Option<Task<ConnectToken>>,
}

/// If we have an io task that is waiting for a `ConnectToken`, we poll the task until completion,
/// then we retrieve the token and update the ClientConfig.
pub fn fetch_connect_token(
    mut connect_token_request: ResMut<ConnectTokenRequestTask>,
    client: Single<(Entity, Has<NetcodeClient>), With<Client>>,
    mut commands: Commands,
) -> Result {
    if let Some(task) = &mut connect_token_request.task {
        if let Some(connect_token) = block_on(future::poll_once(task)) {
            info!("Received ConnectToken, starting connection!");
            info!(
                "Inserting netcode client with our connect token. our client \
                 already has NetcodeClient?: {}",
                client.1
            );
            let client = client.into_inner().0;
            commands.entity(client).insert(NetcodeClient::new(
                Authentication::Token(connect_token),
                NetcodeConfig::default(),
            )?);
            commands.trigger(Connect { entity: client });
            connect_token_request.task = None;
        }
    }
    Ok(())
}

/// Get a ConnectToken via a TCP connection to the authentication server
pub async fn get_connect_token_from_auth_backend(
    auth_backend_address: SocketAddr,
) -> ConnectToken {
    let stream = tokio::net::TcpStream::connect(auth_backend_address)
        .await
        .unwrap_or_else(|_| {
            panic!(
                "Failed to connect to authentication server on {:?}",
                auth_backend_address
            )
        });

    info!("auth backend tcp stream address: {:?}", stream.peer_addr());
    // wait for the socket to be readable
    stream.readable().await.unwrap();
    let mut buffer = [0u8; CONNECT_TOKEN_BYTES];
    match stream.try_read(&mut buffer) {
        Ok(_number_of_bytes_read) => {
            info!(
                "Received token bytes: {:?}. Token len: {:?}",
                buffer,
                buffer.len()
            );
            ConnectToken::try_from_bytes(&buffer)
                .expect("Failed to parse token from authentication server")
        }
        _ => {
            panic!("Failed to read token from authentication server");
        }
    }
}
