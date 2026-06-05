use async_compat::Compat;
use avian3d::prelude::*;
use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use game_core::start_server;
use netvy::prelude::*;
use shared::AppRole;
use shared::character_controller::{
    CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
};
use shared::multiplayer_messages::{ConfirmRespawn, PlayerHitMessage};
use shared::player::Player;
use shared::utils::network::get_dedicated_server_socket_addr_client_side;

// use crate::auth::{
//     ConnectTokenRequestTask, fetch_connect_token,
//     get_connect_token_from_auth_backend,
// };
use crate::character_controller::components::CharacterControllerBundle;
use crate::game_flow::states::{
    AppState, ClientLoadingState, GameModeClient, InGameState,
};

const CLIENT_PORT: u16 = 0;

pub const GENERIC_NO_CONNECTION_ERROR_MESSAGE: &str =
    "Failed to connect to Game Server. Please verify your internet connection \
     works. The Game Server may also be currently unavailable.";

pub struct NetworkPlugin;

// TODO: This plugin; its not really clear what purpose it has. It does too many different things. I
// dont like this.
impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ClientLoadingState::StartingServer),
            start_server,
        );
        app.add_systems(
            OnEnter(ClientLoadingState::ConnectingToServer),
            on_enter_connecting_to_server,
        );
        // app.add_systems(
        //     Update,
        //     fetch_connect_token
        //         .run_if(resource_exists::<ConnectTokenRequestTask>),
        // );
        app.add_systems(
            Update,
            (
                handle_confirm_respawn_message,
                handle_hit_message,
                handle_new_player,
            ),
        );
        app.add_observer(handle_added_server);
        // app.add_observer(handle_disconnect);
    }
}

fn handle_added_server(
    trigger: On<Add, Server>,
    app_role: Res<State<AppRole>>,
    mut next_client_loading_state: ResMut<NextState<ClientLoadingState>>,
) {
    info!("Server {} now added!", trigger.entity);
    if *app_role.get() == AppRole::ClientAndServer {
        info!(
            "Server was added and AppRole::ClientAndServer, setting \
             LoadingState to ConnectingToServer"
        );
        next_client_loading_state.set(ClientLoadingState::ConnectingToServer);
    }
}

fn on_enter_connecting_to_server(
    mut commands: Commands,
    // connected_query: Query<Has<Connected>>,
    game_mode: Res<State<GameModeClient>>,
    server_entity: Query<Entity, With<Server>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    // Connected component only present on our own client
    // for connected in connected_query {
    //     if connected {
    //         warn!("Already connected to the game server");
    //         continue;
    //     }
    // }

    let is_singleplayer = *game_mode.get() != GameModeClient::Multiplayer;

    // FIXME: following code is so insanely unreadable
    if let Ok(server_entity) = server_entity.single()
        && is_singleplayer
    {
        info!("Spawning a host client, we have single player mode");

        let client_entity =
            commands.spawn((Name::new("Host Client"), Client)).id();

        // NOTE: We only trigger the Connect in this system for host client, as the connect for a
        // client connecting to the official dedicated server triggers only when we received a
        // ConnectToken. this happens in auth.rs
        commands.trigger(ConnectToServer { client_entity });
    } else {
        info!(
            "Connecting to official dedicated server, Spawning a 'normal' \
             client"
        );
        // let auth_backend_addr = get_auth_backend_socket_addr_client_side();
        // if let Some(auth_backend_addr) = auth_backend_addr {
        //     debug!(
        //         "Starting task to get auth ConnectToken via AuthBackend at {}",
        //         auth_backend_addr
        //     );
        //     let task = IoTaskPool::get().spawn_local(Compat::new(async move {
        //         get_connect_token_from_auth_backend(auth_backend_addr).await
        //     }));
        //     commands
        //         .insert_resource(ConnectTokenRequestTask { task: Some(task) });
        //     debug!("Inserted ConnectTokenRequestTask");
        // } else {
        //     next_app_state.set(AppState::Disconnected);
        // }

        // if let Some(server_address) =
        //     get_dedicated_server_socket_addr_client_side()
        // {
        let client_entity = commands
            .spawn((
                Name::new("Client"),
                Client,
                TargetAddress {
                    address: "0.0.0.0".to_string(),
                    port: CLIENT_PORT,
                },
            ))
            .id();
        commands.trigger(ConnectToServer { client_entity });
        // } else {
        //     info!(
        //         "Could not resolve server address, setting AppState to \
        //          Disconnected"
        //     );
        //     next_app_state.set(AppState::Disconnected);
        // }
    }
}

fn handle_new_player(
    mut commands: Commands,
    player_query: Query<(Entity, Has<Owned>), Added<Player>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    for (player_entity, has_controlled) in player_query {
        info!("A player was added! {}", player_entity);

        if has_controlled {
            // we insert the character controller locally on our client, as it should only run on the
            // client.
            commands.entity(player_entity).insert((
                CharacterControllerBundle::default(),
                DespawnOnExit(AppState::InGame),
                Visibility::Visible,
                Transform::from_translation(vec3(0.0, 20.0, 0.0)),
                Name::new("Our Player"),
                Mesh3d(meshes.add(Capsule3d::new(
                    CHARACTER_CAPSULE_RADIUS,
                    CHARACTER_CAPSULE_LENGTH,
                ))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: WHITE.into(),
                    ..Default::default()
                })),
            ));

            // TODO: is this a good idea? we assume that if our player is
            // present, it means GameCore is ready.
            info!("Our player was added, setting AppState to InGame");
            next_app_state.set(AppState::InGame);
        } else {
            commands.entity(player_entity).insert((
                Mesh3d(meshes.add(Capsule3d::new(
                    CHARACTER_CAPSULE_RADIUS,
                    CHARACTER_CAPSULE_LENGTH,
                ))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: WHITE.into(),
                    ..Default::default()
                })),
                Name::new("Remote Player"),
                RigidBody::Kinematic,
                Collider::capsule(
                    CHARACTER_CAPSULE_RADIUS,
                    CHARACTER_CAPSULE_LENGTH,
                ),
            ));
        }
    }
}

// FIXME: reimplement
// fn handle_disconnect(
//     trigger: On<Add, Disconnected>,
//     disconnected: Query<&Disconnected>,
//     mut next_app_state: ResMut<NextState<AppState>>,
// ) {
//     match disconnected.get(trigger.entity) {
//         // NOTE: The library inserts Disconnected component per default, as that is the default state,
//         // even if we weren't even connected in the first place. So, for now,
//         // we check if there is a reason, if not, we weren't actually disconnected.
//         // https://github.com/cBournhonesque/lightyear/discussions/1375
//         Ok(disconnected) => {
//             if let Some(disconnected_reason) = &disconnected.reason {
//                 info!(
//                     "Disconnected from server, reason: {:?}",
//                     disconnected_reason
//                 );
//
//                 let parsed_reason =
//                     parse_lightyear_disconnect_reason(disconnected_reason);
//
//                 match parsed_reason {
//                     DisconnectReason::ClientTriggered => {
//                         next_app_state.set(AppState::MainMenu);
//                     }
//                     DisconnectReason::Unknown => {
//                         next_app_state.set(AppState::Disconnected);
//                     }
//                 }
//             }
//         }
//         Err(error) => {
//             warn!(
//                 "Disconnected from server, but could not retrieve \
//                  Disconnected component to get reason for disconnect: {}",
//                 error
//             );
//         }
//     }
// }

fn handle_confirm_respawn_message(
    mut message_receiver: Single<&mut NetMessageReader<ConfirmRespawn>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    for _ in message_receiver.read() {
        info!("Respawn request was confirmed by server!");
        next_in_game_state.set(InGameState::Playing);
    }
}

fn handle_hit_message(
    mut message_receiver: Single<&mut NetMessageReader<PlayerHitMessage>>,
    mut message_sender: MessageWriter<PlayerHitMessage>,
) {
    for message in message_receiver.read() {
        message_sender.write(PlayerHitMessage {
            origin: message.origin,
        });
    }
}
