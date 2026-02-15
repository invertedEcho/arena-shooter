use std::time::Duration;

use avian3d::prelude::*;
use bevy::{
    color::palettes::css::WHITE, platform::collections::HashMap, prelude::*,
};
use lightyear::{
    netcode::NetcodeServer,
    prelude::{
        server::{ClientOf, NetcodeConfig, ServerUdpIo, Start},
        *,
    },
};
use shared::{
    ClientRespawnRequest, ConfirmRespawn, DEFAULT_HEALTH, GameModeServer,
    GameStateServer, HitMessage, SPAWN_POINT_MEDIUM_PLASTIC_MAP, ServerMode,
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
    },
    components::Health,
    enemy::components::Enemy,
    game_score::{GameScore, LivingEntityStats},
    player::{Player, PlayerBundle},
    protocol::{
        ChangeGameServerStateRequest, ClientUpdatePositionMessage,
        EntityPositionServer, OrderedReliableChannel, ShootRequest,
    },
    shooting::MAX_SHOOTING_DISTANCE,
    utils::{
        auth::get_private_key,
        network::{
            NETCODE_PROTOCOL_VERSION, SERVER_SOCKET_ADDR_REMOTE_SERVER,
            SERVER_SOCKET_ADDR_SINGLEPLAYER,
        },
    },
};

use crate::{
    enemy::{
        EnemyPlugin,
        spawn::{EnemySpawnStrategy, SpawnEnemiesMessage},
    },
    game_flow::GameFlowPlugin,
    nav_mesh_pathfinding::NavMeshPathfindingPlugin,
};

mod enemy;
mod game_flow;
mod nav_mesh_pathfinding;

// on client, the state gets reset to Initial when we exit to main menu, as everything gets
// despawned.
// for server binary, this will just be used once, at startup
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum ServerLoadingState {
    #[default]
    Initial,
    GameScoreFinishedSetup,
    MapSpawned,
    CollidersSpawned,
    NavMeshReady,
    Done,
}

#[derive(Resource)]
pub struct GameStateWave {
    pub current_wave: usize,
    pub enemies_killed: usize,
    pub enemies_left_from_current_wave: usize,
}

/// This plugin adds all plugins & systems that need to run on the server, regardless if its for
/// the server binary or the local server that gets started for Singleplayer.
pub struct GameCorePlugin;

impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ServerLoadingState>();
        app.init_state::<GameStateServer>();

        app.add_plugins(lightyear::prelude::server::ServerPlugins::default());

        app.add_plugins(EnemyPlugin);
        app.add_plugins(NavMeshPathfindingPlugin);
        app.add_plugins(GameFlowPlugin);

        app.add_systems(Startup, start_server);

        app.add_systems(
            Update,
            (
                handle_shoot_requests,
                receive_client_update_position,
                handle_client_respawn_requests,
                handle_game_server_state_update_request,
            ),
        );

        app.add_systems(
            OnEnter(ServerLoadingState::Done),
            handle_server_loading_state_done,
        );

        app.add_systems(OnEnter(ServerLoadingState::Initial), setup_game_score);

        app.add_observer(handle_new_connection);
        app.add_observer(spawn_player_on_new_client);
    }
}

pub fn start_server(
    mut commands: Commands,
    server_mode: Res<State<ServerMode>>,
) {
    let entity_name = match server_mode.get() {
        ServerMode::LocalServerSinglePlayer => "Local Server for singleplayer",
        ServerMode::RemoteServer => "Server from server Binary",
    };

    let local_addr =
        if *server_mode.get() == ServerMode::LocalServerSinglePlayer {
            SERVER_SOCKET_ADDR_SINGLEPLAYER
        } else {
            SERVER_SOCKET_ADDR_REMOTE_SERVER
        };

    info!("Starting server on {}", local_addr);

    let server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig {
                protocol_id: NETCODE_PROTOCOL_VERSION,
                private_key: get_private_key(&server_mode),
                ..default()
            }),
            LocalAddr(local_addr),
            ServerUdpIo::default(),
            Name::new(entity_name),
            GameModeServer::FreeForAll,
        ))
        .id();

    commands.trigger(Start { entity: server });
}

fn setup_game_score(
    mut commands: Commands,
    mut next_server_loading_state: ResMut<NextState<ServerLoadingState>>,
    server_mode: Res<State<ServerMode>>,
) {
    commands
        .spawn((
            GameScore {
                players: HashMap::new(),
                enemies: HashMap::new(),
            },
            Name::new("Game Score"),
        ))
        .insert_if(Replicate::to_clients(NetworkTarget::All), || {
            *server_mode.get() == ServerMode::RemoteServer
        });

    // NOTE: theoretically the game score entity is not necessarily already spawned here, but we
    // just do it here as spawning such a simple entity is trivial.
    next_server_loading_state.set(ServerLoadingState::GameScoreFinishedSetup);
}

fn handle_new_connection(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert((ReplicationSender::new(
            Duration::from_millis(100),
            SendUpdatesMode::SinceLastAck,
            false,
        ),));
}

fn spawn_player_on_new_client(
    trigger: On<Add, Connected>,
    clients_query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    server_mode: Res<State<ServerMode>>,
    mut game_score: Query<&mut GameScore>,
) {
    if let Ok(remote_id) = clients_query.get(trigger.entity) {
        let peer_id = remote_id.0;

        let mut game_score = game_score.single_mut().unwrap();

        game_score.players.insert(
            peer_id.to_bits(),
            LivingEntityStats {
                username: format!("Player {}", peer_id.to_bits()),
                ..default()
            },
        );

        info!(
            "Spawning a player for fully connected Client entity: {} | \
             peer_id: {}",
            trigger.entity, peer_id
        );

        // NOTE: The replicate component gets inserted into the player entity, but only registered
        // components will be replicated to all other clients
        let player_entity = commands
            .spawn((
                PlayerBundle::default(),
                Name::new("Player"),
                Replicate::to_clients(NetworkTarget::All),
                // TODO: think we could override replication behaviour for this component and only
                // replicate to all other clients than the current client
                EntityPositionServer {
                    translation: vec3(0.0, 20.0, 0.0),
                },
                Visibility::Visible,
                // we add the ControlledBy on the server, with the client entity as the owner of this
                // player, so on the client we can then filter by players that have the `Controlled`
                // component and those are the players that are actually owned by that client
                ControlledBy {
                    owner: trigger.entity,
                    lifetime: Lifetime::SessionBased,
                },
                Collider::capsule(
                    CHARACTER_CAPSULE_RADIUS,
                    CHARACTER_CAPSULE_LENGTH,
                ),
                RigidBody::Kinematic,
            ))
            .id();

        if *server_mode == ServerMode::RemoteServer {
            // on headless setup, materials doesnt exist
            if let Some(mut materials) = materials {
                commands.entity(player_entity).insert((
                    Mesh3d(meshes.add(Capsule3d::new(
                        CHARACTER_CAPSULE_RADIUS,
                        CHARACTER_CAPSULE_LENGTH,
                    ))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: WHITE.into(),
                        ..Default::default()
                    })),
                ));
            }
        }
        if *server_mode == ServerMode::LocalServerSinglePlayer {
            commands.entity(player_entity).insert(Controlled);
        }
    }
}

/// This systems receives a message from clients, that their position has changed.
/// The server will then apply it to the `PlayerPositionServer` component, which then gets
/// replicated to all clients. All clients receive the updates from `PlayerPositionServer`, and
/// update the Transform locally.
fn receive_client_update_position(
    mut receivers: Query<(
        &mut MessageReceiver<ClientUpdatePositionMessage>,
        Entity,
    )>,
    mut players: Query<
        (&mut EntityPositionServer, &mut Transform, &ControlledBy),
        With<Player>,
    >,
) {
    for (mut message_receiver, remote_id) in receivers.iter_mut() {
        for message in message_receiver.receive() {
            if let Some((mut server_position, mut transform, _)) = players
                .iter_mut()
                .find(|(_, _, controlled_by)| controlled_by.owner == remote_id)
            {
                server_position.translation = message.new_translation;
                transform.translation = message.new_translation;
            } else {
                warn!(
                    "Received a ClientUpdatePositionMessage but couldnt find \
                     the corresponding Player entity on the server"
                );
            }
        }
    }
}

fn handle_shoot_requests(
    mut commands: Commands,
    receivers: Query<(&mut MessageReceiver<ShootRequest>, Entity, &RemoteId)>,
    mut health_query: Query<&mut Health>,
    spatial_query: SpatialQuery,
    player_query: Query<(Entity, &ControlledBy), With<Player>>,
    mut game_score: Single<&mut GameScore>,
    game_mode_server: Single<&GameModeServer>,
    client_query: Query<&RemoteId, With<ClientOf>>,
    mut server_multi_message_sender: ServerMultiMessageSender,
    server: Single<&Server>,
) {
    for (mut message_receiver, client_entity_server_side, remote_id) in
        receivers
    {
        for message in message_receiver.receive() {
            let Some(shooter_entity) = player_query
                .iter()
                .find(|(_, controlled_by)| {
                    controlled_by.owner == client_entity_server_side
                })
                .map(|i| i.0)
            else {
                warn!(
                    "Received a ShootRequest but couldn't determine from \
                     which player this came from"
                );
                continue;
            };

            let Some(first_hit) = spatial_query.cast_ray(
                message.origin,
                message.direction,
                MAX_SHOOTING_DISTANCE,
                false,
                &SpatialQueryFilter::default()
                    .with_excluded_entities([shooter_entity]),
            ) else {
                continue;
            };

            if let Ok(mut health) = health_query.get_mut(first_hit.entity) {
                health.0 -= 8.0;
                if let Ok(client_entity_that_was_hit) =
                    player_query.get(first_hit.entity).map(|i| i.1)
                    && let Ok(client) =
                        client_query.get(client_entity_that_was_hit.owner)
                {
                    server_multi_message_sender
                        .send::<HitMessage, OrderedReliableChannel>(
                            &HitMessage {
                                origin: message.origin,
                            },
                            &server,
                            &NetworkTarget::Single(client.0),
                        )
                        .ok();
                } else {
                    error!("Could not find client that was hit by the bullet");
                }

                if health.0 <= 0.0 {
                    let entity_killed = first_hit.entity;
                    commands.entity(entity_killed).insert(ColliderDisabled);

                    match game_score.players.get_mut(&remote_id.to_bits()) {
                        Some(player) => {
                            info!(
                                "increased kill count of player with \
                                 remote_id: {}",
                                remote_id.to_bits()
                            );
                            player.kills += 1;
                        }
                        None => {
                            warn!(
                                "Failed to find player in game score by \
                                 remote_id {}\nGame score: {:?}",
                                remote_id.to_bits(),
                                *game_score
                            )
                        }
                    }

                    // if we have game mode wave, the entity killed will always be an enemy. so we
                    // skip this case
                    if **game_mode_server == GameModeServer::Waves {
                        return;
                    };
                    match player_query.get(entity_killed) {
                        Ok((_, controlled_by)) => {
                            if let Ok(remote_id) =
                                client_query.get(controlled_by.owner)
                                && let Some(player_score) = game_score
                                    .players
                                    .get_mut(&remote_id.to_bits())
                            {
                                player_score.deaths += 1;
                            } else {
                                warn!(
                                    "Failed to find client of player that was \
                                     killed"
                                );
                            };
                        }
                        Err(error) => {
                            warn!(
                                "Failed to find player that was killed: {}",
                                error
                            );
                        }
                    }
                }
            }
        }
    }
}

fn handle_server_loading_state_done(
    mut commands: Commands,
    game_mode_server: Single<&GameModeServer>,
    mut spawn_enemies: MessageWriter<SpawnEnemiesMessage>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    info!(
        "ServerLoadingState is done, now doing actions corresponding to game \
         mode. Game mode is: {:?}",
        *game_mode_server
    );

    match *game_mode_server {
        GameModeServer::Waves => {
            commands.insert_resource(GameStateWave {
                current_wave: 1,
                enemies_killed: 0,
                enemies_left_from_current_wave: 3,
            });
            spawn_enemies.write(SpawnEnemiesMessage {
                // FIXME: revert
                enemy_count: 1,
                spawn_strategy: EnemySpawnStrategy::RandomSelection,
            });
        }
        GameModeServer::FreeForAll | GameModeServer::FreeRoam => {
            commands.remove_resource::<GameStateWave>();
            for enemy in enemy_query {
                commands.entity(enemy).despawn();
            }
        }
    };
}

fn handle_client_respawn_requests(
    mut commands: Commands,
    receivers: Query<(
        &mut MessageReceiver<ClientRespawnRequest>,
        &ControlledByRemote,
        &RemoteId,
    )>,
    mut player_query: Query<(Entity, &mut Health, &mut EntityPositionServer)>,
    mut server_multi_message_sender: ServerMultiMessageSender,
    server: Single<&Server>,
) {
    for (mut message_receiver, controlled_by, remote_id) in receivers {
        for _ in message_receiver.receive() {
            info!("Received ClientRespawnRequest!");
            match controlled_by.iter().next() {
                Some(controlling_player) => {
                    match player_query.get_mut(controlling_player) {
                        Ok((
                            player_entity,
                            mut player_health,
                            mut entity_position_server,
                        )) => {
                            player_health.0 = DEFAULT_HEALTH;
                            entity_position_server.translation =
                                SPAWN_POINT_MEDIUM_PLASTIC_MAP;

                            commands
                                .entity(player_entity)
                                .remove::<ColliderDisabled>();

                            info!(
                                "Sending confirm respawn message to client \
                                 with remote_id: {}",
                                remote_id.0
                            );
                            let network_target =
                                &NetworkTarget::Single(remote_id.0);

                            let message_sent_result = server_multi_message_sender
                                .send::<ConfirmRespawn, OrderedReliableChannel>(
                                    &ConfirmRespawn,
                                    &server,
                                    network_target,
                                );
                            match message_sent_result {
                                Ok(_) => {
                                    info!(
                                        "Succesfully sent ConfirmRespawn \
                                         message to client"
                                    );
                                }
                                Err(error) => {
                                    error!(
                                        "Failed to send ConfirmRespawn \
                                         message to client: {}",
                                        error
                                    );
                                }
                            }
                        }
                        Err(error) => {
                            warn!(
                                "Failed to find controlling player: {}",
                                error
                            );
                        }
                    }
                }
                None => {
                    warn!(
                        "Received a ClientRespawnRequest but no \
                         'ControlledByRemote' exists"
                    );
                }
            }
        }
    }
}

fn handle_game_server_state_update_request(
    mut message_receiver: Single<
        &mut MessageReceiver<ChangeGameServerStateRequest>,
    >,
    server_mode: Res<State<ServerMode>>,
    mut game_state_server: ResMut<NextState<GameStateServer>>,
) {
    for message in message_receiver.receive() {
        if *server_mode.get() != ServerMode::LocalServerSinglePlayer {
            info!("Ignored ChangeGameServerStateRequest");
            return;
        }

        info!("GameStateServer updated to {:?}", message.0);
        game_state_server.set(message.0);
    }
}
