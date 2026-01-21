use std::net::{Ipv6Addr, SocketAddr};

use async_compat::Compat;
use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use shared::player::Player;
use shared::protocol::{
    ClientUpdatePositionMessage, OrderedReliableMessageChannel,
    PlayerPositionServer,
};
use shared::utils::lightyear::{
    DisconnectReason, parse_lightyear_disconnect_reason,
};
use shared::{
    NETCODE_PROTOCOL_VERSION, SERVER_PORT, ServerMode, get_private_key,
    get_server_socket_addr_client_side,
};

use crate::auth::{
    ConnectTokenRequestTask, fetch_connect_token,
    get_connect_token_from_auth_backend,
};
use crate::character_controller::components::CharacterControllerBundle;
use crate::game_flow::game_mode::GameModeState;
use crate::game_flow::states::{
    AppState, DisconnectedState, InGameState, LoadingGameState,
};
use crate::player::camera::messages::SpawnPlayerCamerasMessage;

const CLIENT_PORT: u16 = 0;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(LoadingGameState::ConnectingToServer),
            on_enter_connecting_to_server,
        );
        app.add_systems(
            Update,
            (
                send_client_update_position.run_if(in_state(AppState::InGame)),
                apply_server_position_other_clients,
                fetch_connect_token,
            ),
        );
        app.add_observer(handle_new_player);
        app.add_observer(handle_connected);
        app.add_observer(handle_disconnect);
    }
}

pub fn on_enter_connecting_to_server(
    mut commands: Commands,
    connected_query: Query<Has<Connected>>,
    mut task_state: ResMut<ConnectTokenRequestTask>,
    game_mode: Res<State<GameModeState>>,
    server_entity: Query<Entity, With<Server>>,
    server_mode: Res<ServerMode>,
) {
    // Connected component only present on our own client
    for connected in connected_query {
        if connected {
            info!("Already connected, skipping ConnectToServerMessage");
            continue;
        }
    }

    let is_multiplayer = *game_mode.get() == GameModeState::Multiplayer;

    if is_multiplayer {
        let auth_backend_addr = task_state.auth_backend_addr;
        debug!(
            "Starting task to get auth ConnectToken via AuthBackend at {}",
            auth_backend_addr
        );
        let task = IoTaskPool::get().spawn_local(Compat::new(async move {
            get_connect_token_from_auth_backend(auth_backend_addr).await
        }));
        task_state.task = Some(task);
    }

    let server_address = match *game_mode.get() {
        GameModeState::Multiplayer => get_server_socket_addr_client_side(),
        _ => SocketAddr::new(
            std::net::IpAddr::V6(Ipv6Addr::LOCALHOST),
            SERVER_PORT,
        ),
    };

    let private_key = get_private_key(&server_mode);

    if let Ok(server_entity) = server_entity.single()
        && !is_multiplayer
    {
        let client = commands
            .spawn((
                Name::new("Host Client"),
                LinkOf {
                    server: server_entity,
                },
            ))
            .id();

        commands.entity(client).insert(
            NetcodeClient::new(
                Authentication::Manual {
                    server_addr: server_address,
                    client_id: 0,
                    protocol_id: NETCODE_PROTOCOL_VERSION,
                    private_key,
                },
                NetcodeConfig::default(),
            )
            .unwrap(),
        );

        commands.trigger(Connect { entity: client });
    } else {
        let client = commands
            .spawn((
                Name::new("Client"),
                Client::default(),
                LocalAddr(SocketAddr::new(
                    Ipv6Addr::UNSPECIFIED.into(),
                    CLIENT_PORT,
                )),
                PeerAddr(server_address),
                Link::new(None),
                ReplicationReceiver::default(),
                UdpIo::default(),
            ))
            .id();
        commands.trigger(Connect { entity: client });
    }
}

fn handle_connected(
    _trigger: On<Add, Connected>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    debug!("Connected to server, setting AppState to InGame");
    next_app_state.set(AppState::InGame);
}

fn handle_new_player(
    trigger: On<Add, Player>,
    mut commands: Commands,
    player_query: Query<(Entity, Has<Controlled>), With<Player>>,
    mut spawn_player_camera_message_writer: MessageWriter<
        SpawnPlayerCamerasMessage,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    server_mode: Res<ServerMode>,
) {
    let Ok((our_player_entity, has_controlled)) =
        player_query.get(trigger.entity)
    else {
        return;
    };

    let remote_server_and_our_player =
        *server_mode == ServerMode::RemoteServer && has_controlled;
    if remote_server_and_our_player
        || *server_mode == ServerMode::LocalServerSinglePlayer
    {
        // we insert the character controller locally on our client, as it should only run on the
        // client. as it is not registered in our protocol, it wont be replicated.
        commands.entity(our_player_entity).insert((
            CharacterControllerBundle::default(),
            DespawnOnExit(AppState::InGame),
            Visibility::Visible,
            Transform::from_translation(vec3(0.0, 20.0, 0.0)),
            Name::new("Our Player"),
        ));

        spawn_player_camera_message_writer
            .write(SpawnPlayerCamerasMessage(trigger.entity));
    } else if *server_mode == ServerMode::RemoteServer && !has_controlled {
        commands.entity(trigger.entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(0.2, 1.3))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: WHITE.into(),
                ..Default::default()
            })),
            Name::new("Remote Player"),
        ));
    }
}

pub fn send_client_update_position(
    // With<Client> also ensures its the messagesender from our local client, as client component
    // only gets inserted into our own client
    mut message_sender: Single<
        &mut MessageSender<ClientUpdatePositionMessage>,
        With<Client>,
    >,
    player_transform: Single<&Transform, (With<Player>, With<Controlled>)>,
) {
    message_sender.send::<OrderedReliableMessageChannel>(
        ClientUpdatePositionMessage {
            new_translation: player_transform.translation,
        },
    );
}

pub fn apply_server_position_other_clients(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Transform,
            &PlayerPositionServer,
            Has<Controlled>, // present only on own player
        ),
        With<Player>,
    >,
) {
    for (mut transform, server_pos, controlled) in &mut query {
        // Skip our own transform
        if controlled {
            continue;
        }

        let target = server_pos.translation;
        let current = transform.translation;

        let lerp_factor = 10.0 * time.delta_secs();
        transform.translation = current.lerp(target, lerp_factor);
    }
}

pub fn handle_disconnect(
    trigger: On<Add, Disconnected>,
    disconnected: Query<&Disconnected>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
    mut next_disconnected_state: ResMut<NextState<DisconnectedState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    match disconnected.get(trigger.entity) {
        // NOTE: The library inserts Disconnected component per default, as that is the default state,
        // even if we weren't even connected in the first place. So, for now,
        // we check if there is a reason, if not, we weren't actually disconnected.
        // https://github.com/cBournhonesque/lightyear/discussions/1375
        Ok(disconnected) => {
            if let Some(disconnected_reason) = &disconnected.reason {
                info!(
                    "Disconnected from server, reason: {:?}",
                    disconnected_reason
                );

                let parsed_reason =
                    parse_lightyear_disconnect_reason(disconnected_reason);

                match parsed_reason {
                    DisconnectReason::ClientTriggered => {
                        next_app_state.set(AppState::MainMenu);
                    }
                    DisconnectReason::Unknown => {
                        // TODO: we also need to set to InGame, in case we are on initial connecting, e.g.
                        // LoadingScreen -> failed to connect -> wouldnt be in ingame, but in
                        // AppState::LoadingGame, so we set InGame here to be safe as InGameState only
                        // exists when AppState is InGame
                        next_app_state.set(AppState::InGame);

                        next_in_game_state.set(InGameState::Disconnected);
                        next_disconnected_state.set(DisconnectedState::Reason(
                            disconnected_reason.to_string(),
                        ));
                    }
                }
            }
        }
        Err(error) => {
            warn!(
                "Disconnected from server, but could not retrieve \
                 Disconnected component to get reason for disconnect: {}",
                error
            );
        }
    }
}
