use std::net::SocketAddr;

use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{Task, block_on};
use lightyear::netcode::{CONNECT_TOKEN_BYTES, ConnectToken};
use lightyear::prelude::client::*;
use lightyear::prelude::*;

use crate::game_flow::states::{AppState, ServerConnectionState};

/// Holds a handle to an io task that is requesting a `ConnectToken` from the backend
#[derive(Resource)]
pub struct ConnectTokenRequestTask {
    pub auth_backend_addr: SocketAddr,
    pub task: Option<Task<Option<ConnectToken>>>,
}

/// If we have an io task that is waiting for a `ConnectToken`, we poll the task until completion,
/// then we retrieve the token and update the ClientConfig.
pub fn fetch_connect_token(
    mut connect_token_request: ResMut<ConnectTokenRequestTask>,
    client: Single<Entity, With<Client>>,
    mut commands: Commands,
    mut next_disconnected_state: ResMut<NextState<ServerConnectionState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if let Some(task) = &mut connect_token_request.task
        && let Some(connect_token) = block_on(future::poll_once(task))
    {
        let client = client.into_inner();

        let Some(connect_token) = connect_token else {
            warn!("ConnectToken is None, couldnt connect to game server");
            connect_token_request.task = None;
            next_disconnected_state.set(ServerConnectionState::Disconnected);
            next_app_state.set(AppState::Disconnected);
            return;
        };

        info!("Received ConnectToken, starting connection!");
        info!("Inserting netcode client with our connect token",);

        commands.entity(client).insert(
            NetcodeClient::new(
                Authentication::Token(connect_token),
                NetcodeConfig::default(),
            )
            .unwrap(),
        );
        commands.trigger(Connect { entity: client });
        connect_token_request.task = None;
    }
}

/// Get a ConnectToken via a TCP connection to the authentication server
pub async fn get_connect_token_from_auth_backend(
    auth_backend_address: SocketAddr,
) -> Option<ConnectToken> {
    let Ok(stream) = tokio::net::TcpStream::connect(auth_backend_address).await
    else {
        return None;
    };

    info!("auth backend tcp stream address: {:?}", stream.peer_addr());
    // wait for the socket to be readable
    stream.readable().await.unwrap();
    let mut buffer = [0u8; CONNECT_TOKEN_BYTES];
    match stream.try_read(&mut buffer) {
        Ok(n) if n == CONNECT_TOKEN_BYTES => {
            match ConnectToken::try_from_bytes(&buffer) {
                Ok(connect_token) => Some(connect_token),
                Err(error) => {
                    error!(
                        "Failed to parse ConnectToken from authentication \
                         server. {}",
                        error
                    );
                    return None;
                }
            }
        }
        _ => {
            return None;
        }
    };
    None
}
