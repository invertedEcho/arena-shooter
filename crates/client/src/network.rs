use avian3d::prelude::*;
use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use netvy::prelude::*;
use shared::character_controller::{
    CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
};
use shared::multiplayer_messages::{ConfirmRespawn, PlayerHitMessage};
use shared::player::Player;
use shared::utils::network::SERVER_PORT;

// use crate::auth::{
//     ConnectTokenRequestTask, fetch_connect_token,
//     get_connect_token_from_auth_backend,
// };
use crate::character_controller::components::CharacterControllerBundle;
use crate::game_flow::states::{AppState, ClientLoadingState, InGameState};

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
            start_host_client_server,
        );
        // app.add_systems(
        //     Update,
        //     fetch_connect_token
        //         .run_if(resource_exists::<ConnectTokenRequestTask>),
        // );
        app.add_systems(
            FixedUpdate,
            (
                handle_confirm_respawn_message,
                handle_hit_message,
                handle_new_player,
                handle_added_owned_player,
                spawn_host_client,
            ),
        );
        // app.add_observer(handle_added_server);
        // app.add_observer(handle_disconnect);
    }
}

fn handle_new_player(
    mut commands: Commands,
    player_query: Query<Entity, Added<Player>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for player_entity in player_query {
        info!(
            "A player was added! Spawning visuals (entity={})",
            player_entity
        );

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

fn handle_added_owned_player(
    mut commands: Commands,
    player_entity: Single<Entity, (With<Player>, Added<Owned>)>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    // we insert the character controller locally on our client, as it should only run on the
    // client.
    commands.entity(*player_entity).insert((
        CharacterControllerBundle::default(),
        DespawnOnExit(AppState::InGame),
        Visibility::Visible,
        Transform::from_translation(vec3(0.0, 20.0, 0.0)),
        Name::new("Our Player"),
    ));

    // TODO: is this a good idea? we assume that if our player is
    // present, it means GameCore is ready.
    info!("Our player was added, setting AppState to InGame");
    next_app_state.set(AppState::InGame);
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

/// marker component for host client server
#[derive(Component)]
struct HostClientServer;

fn start_host_client_server(mut commands: Commands) {
    info!("Starting HostClient server");
    let server_entity = commands
        .spawn((
            Server,
            TargetAddress {
                address: "127.0.0.1".to_string(),
                port: SERVER_PORT,
            },
            HostClientServer,
        ))
        .id();

    commands.trigger(StartServer { server_entity });
}

fn spawn_host_client(
    mut commands: Commands,
    added_host_client_server: Query<Entity, Added<HostClientServer>>,
    mut next_client_connection_state: ResMut<NextState<ClientLoadingState>>,
) {
    for _ in added_host_client_server {
        next_client_connection_state
            .set(ClientLoadingState::ConnectingToServer);
        let client_entity = commands
            .spawn((
                Client,
                TargetAddress {
                    address: "127.0.0.1".to_string(),
                    port: SERVER_PORT,
                },
            ))
            .id();

        commands.trigger(ConnectToServer { client_entity });
    }
}
