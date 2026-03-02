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
    AppRole, ClientRespawnRequest, ConfirmRespawn, DEFAULT_HEALTH,
    GameCoreReady, GameModeServer, GameStateServer, MEDIUM_PLASTIC_MAP_PATH,
    PlayerHitMessage, SPAWN_POINT_MEDIUM_PLASTIC_MAP, SelectedMapState,
    StartSinglePlayerGame, StopSinglePlayerGame,
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
        auth::{LOCAL_SERVER_PRIVATE_KEY, load_private_key_from_env},
        network::{
            NETCODE_PROTOCOL_VERSION, SERVER_SOCKET_ADDR_DEDICATED_SERVER,
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
pub enum GameCoreLoadingState {
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
        app.init_state::<GameCoreLoadingState>();
        app.init_state::<GameStateServer>();
        app.init_state::<SelectedMapState>();

        app.add_plugins(lightyear::prelude::server::ServerPlugins::default());

        app.add_plugins(EnemyPlugin);

        app.add_plugins(NavMeshPathfindingPlugin);

        app.add_plugins(GameFlowPlugin);

        app.add_systems(
            Update,
            (
                handle_shoot_requests,
                receive_client_update_position,
                handle_client_respawn_requests,
                handle_game_server_state_update_request,
                kill_players_below_death_zone,
                read_stop_single_player_game,
            ),
        );

        app.add_systems(
            OnEnter(GameCoreLoadingState::Done),
            on_game_core_loading_state_done,
        );

        // now we dont start loading at startup but only when we receive the StartGameRequest from
        // the client
        app.add_systems(Update, handle_start_single_player_game);

        app.add_systems(
            OnEnter(GameCoreLoadingState::GameScoreFinishedSetup),
            on_enter_spawn_map,
        );

        app.add_observer(handle_new_connection);
        app.add_observer(spawn_player_on_new_client);
        app.add_observer(check_collider_constructor_hierarchy_ready);

        app.add_systems(
            Update,
            log_updates_to_game_core_loading_state
                .run_if(state_changed::<GameCoreLoadingState>),
        );
    }
}

pub fn start_server(mut commands: Commands, app_role: Res<State<AppRole>>) {
    let entity_name = match app_role.get() {
        AppRole::ClientOnly => {
            info!("Skipping starting of server, AppRole is ClientOnly");
            return;
        }
        AppRole::ClientAndServer => "Local Server for singleplayer",
        AppRole::DedicatedServer => "Server from server Binary",
    };

    let local_addr = match app_role.get() {
        AppRole::ClientOnly => {
            return;
        }
        AppRole::ClientAndServer => SERVER_SOCKET_ADDR_SINGLEPLAYER,
        AppRole::DedicatedServer => SERVER_SOCKET_ADDR_DEDICATED_SERVER,
    };

    info!(
        "Starting server on {}, current AppRole: {:?}",
        local_addr,
        app_role.get()
    );

    let private_key = match app_role.get() {
        AppRole::ClientOnly => {
            return;
        }
        AppRole::ClientAndServer => LOCAL_SERVER_PRIVATE_KEY,
        AppRole::DedicatedServer => load_private_key_from_env().unwrap(),
    };

    let server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig {
                protocol_id: NETCODE_PROTOCOL_VERSION,
                private_key,
                ..default()
            }),
            LocalAddr(local_addr),
            ServerUdpIo::default(),
            Name::new(entity_name),
            GameModeServer::FreeForAll,
            DespawnOnExit(AppRole::ClientAndServer),
        ))
        .id();

    commands.trigger(Start { entity: server });
}

// FIXME: this function needs to also run / StartSinglePlayerGame message also needs to be written
// in server binary at startup
fn handle_start_single_player_game(
    mut commands: Commands,
    mut next_server_loading_state: ResMut<NextState<GameCoreLoadingState>>,
    app_role: Res<State<AppRole>>,
    mut message_receiver: MessageReader<StartSinglePlayerGame>,
) {
    for _ in message_receiver.read() {
        info!("RECEIVED StartSinglePlayerGame!!!!");
        commands
            .spawn((
                GameScore {
                    players: HashMap::new(),
                    enemies: HashMap::new(),
                },
                Name::new("Game Score"),
            ))
            .insert_if(Replicate::to_clients(NetworkTarget::All), || {
                *app_role.get() == AppRole::DedicatedServer
            });

        // NOTE: theoretically the game score entity is not necessarily already spawned here, but we
        // just do it here as spawning such a simple entity is trivial.
        info!("SPAWNED GAME SCORE, SETTING GameScoreFinishedSetup!");
        next_server_loading_state
            .set(GameCoreLoadingState::GameScoreFinishedSetup);
    }
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
    mut game_score: Query<&mut GameScore>,
    app_role: Res<State<AppRole>>,
) {
    if let Ok(remote_id) = clients_query.get(trigger.entity) {
        let peer_id = remote_id.0;

        // why does game score not exist here yet?
        if let Ok(mut game_score) = game_score.single_mut() {
            game_score.players.insert(
                peer_id.to_bits(),
                LivingEntityStats {
                    username: format!("Player {}", peer_id.to_bits()),
                    ..default()
                },
            );
        } else {
            error!(
                "No game score currently exists, player will be missing in \
                 game score!"
            );
        }

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

        if *app_role.get() == AppRole::DedicatedServer {
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
        if *app_role.get() == AppRole::ClientAndServer {
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
    enemy_query: Query<Entity, With<Enemy>>,
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
                let is_enemy = enemy_query.get(first_hit.entity).is_ok();
                if !is_enemy {
                    if let Ok(client_entity_that_was_hit) =
                        player_query.get(first_hit.entity).map(|i| i.1)
                        && let Ok(client) =
                            client_query.get(client_entity_that_was_hit.owner)
                    {
                        server_multi_message_sender
                            .send::<PlayerHitMessage, OrderedReliableChannel>(
                                &PlayerHitMessage {
                                    origin: message.origin,
                                },
                                &server,
                                &NetworkTarget::Single(client.0),
                            )
                            .ok();
                    } else {
                        error!(
                            "Could not find client that was hit by the bullet"
                        );
                    }
                }

                if health.0 <= 0.0 {
                    let entity_killed = first_hit.entity;
                    commands.entity(entity_killed).insert(ColliderDisabled);

                    match game_score.players.get_mut(&remote_id.to_bits()) {
                        Some(player) => {
                            debug!(
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

fn on_game_core_loading_state_done(
    mut commands: Commands,
    game_mode_server: Single<&GameModeServer>,
    mut spawn_enemies: MessageWriter<SpawnEnemiesMessage>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    info!(
        "GameCoreLoadingState is done, now doing actions corresponding to \
         game mode. Game mode is: {:?}",
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
                enemy_count: 3,
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
    app_role: Res<State<AppRole>>,
    mut game_state_server: ResMut<NextState<GameStateServer>>,
) {
    for message in message_receiver.receive() {
        if *app_role.get() != AppRole::ClientAndServer {
            info!("Ignored ChangeGameServerStateRequest");
            return;
        }

        info!("GameStateServer updated to {:?}", message.0);
        game_state_server.set(message.0);
    }
}

fn kill_players_below_death_zone(
    player_query: Query<(&mut Health, &Transform), With<Player>>,
) {
    const DEATH_ZONE: f32 = -30.0;
    for (mut health, transform) in player_query {
        if transform.translation.y < DEATH_ZONE && health.0 > 0.0 {
            info!("A player is lower than y = -30, killing");
            health.0 = 0.0;
        }
    }
}

// FIXME: hmm i mean this is weird?
// 1. AppRole::ClientOnly: yes, we want this to run, to know whether colliders have spawned locally
//    of local map. but there we dont want to change ServerLoadingState?
// 2. AppRole::ClientAndServer: here, it makes sense. we change the ServerLoadingState
// 3. AppRole::DedicatedServer: here, it also makes sense. we can change the ServerLoadingState
// TODO: we now have multiple colliderconstructor hierarchies. we need to compare count of ready
// events with expected
fn check_collider_constructor_hierarchy_ready(
    _trigger: On<ColliderConstructorHierarchyReady>,
    current_loading_state: Res<State<GameCoreLoadingState>>,
    mut next_server_loading_state: ResMut<NextState<GameCoreLoadingState>>,
    mut already_done: Local<bool>,
) {
    if *already_done {
        info!("ALREADY DONE, SKIPPING!");
        return;
    }

    if *current_loading_state.get() != GameCoreLoadingState::CollidersSpawned {
        info!(
            "ColliderConstructorHierarchyReady!, setting \
             ServerLoadingState::CollidersSpawned"
        );

        next_server_loading_state.set(GameCoreLoadingState::CollidersSpawned);
        *already_done = true;
    }
}

/// We store the world scene handle as we listen for any AssetEvents::Scene to be
/// `LoadedWithDependencies`. but this of course gets triggered for any scenes that get spawned,
/// like enemy models, so we need to compare our WorldSceneHandle that we insert when we spawn the
/// map with the one that we get from the LoadedWithDependencies message/event.
#[derive(Resource)]
pub struct WorldSceneHandle(pub Handle<Scene>);

pub fn check_world_scene_loaded(
    mut asset_event_message_reader: MessageReader<AssetEvent<Scene>>,
    mut next_game_core_loading_state: ResMut<NextState<GameCoreLoadingState>>,
    maybe_world_scene_handle: Option<Res<WorldSceneHandle>>,
) {
    for asset_event in asset_event_message_reader.read() {
        if let AssetEvent::LoadedWithDependencies { id } = asset_event
            && let Some(ref world_scene_handle) = maybe_world_scene_handle
            && *id == world_scene_handle.0.id()
        {
            info!(
                "Map fully spawned, setting LoadingGameSubState to \
                 SpawningColliders"
            );
            next_game_core_loading_state.set(GameCoreLoadingState::MapSpawned);
        }
    }
}

#[derive(Component)]
struct GameMap;

#[derive(Component)]
struct GameMapLight;

/// Spawns the corresponding map (determined by looking at SelectedMapState) on the client, when
/// we enter LoadingGameState::SpawningMap
fn on_enter_spawn_map(asset_server: Res<AssetServer>, mut commands: Commands) {
    // FIXME: reintroduce choosing different map on client.
    // i think im gonna do that with a request message that can be sent from client to server.
    // thats probably also very future proof in case we want to allow changing map while server is
    // running etc
    let map_path = MEDIUM_PLASTIC_MAP_PATH;

    info!("Spawning the game map");
    commands.spawn((
        DirectionalLight {
            illuminance: 6000.,
            shadows_enabled: true,
            ..default()
        },
        GameMapLight,
        Transform::default().looking_at(Vec3::new(-1.0, -3.0, -2.0), Vec3::Y),
        // FIXME: reintroduce. this should be inserted on client
        // or can we just insert this always?
        // TODO: should be constants
        // RenderLayers::from_layers(&[0, 1]),
    ));

    let world_scene_handle =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(map_path));

    commands.insert_resource(WorldSceneHandle(world_scene_handle.clone()));

    commands.spawn((
        GameMap,
        SceneRoot(world_scene_handle),
        Name::new("Scene Root (Map)"),
        Visibility::Visible,
        RigidBody::Static,
    ));
}

fn log_updates_to_game_core_loading_state(
    mut commands: Commands,
    game_core_loading_state: Res<State<GameCoreLoadingState>>,
    server: Single<Entity, With<Server>>,
) {
    info!("\n");
    info!(
        "GameCoreLoadingState UPDATED! Now: {:?}",
        *game_core_loading_state.get()
    );
    info!("\n");
    if *game_core_loading_state.get() == GameCoreLoadingState::Done {
        commands.entity(*server).insert(GameCoreReady);
    }
}

type EntitiesToDespawnQueryFilter = Or<(
    With<GameMap>,
    With<GameMapLight>,
    With<Enemy>,
    With<Server>,
    With<Client>,
    With<Player>,
)>;

fn read_stop_single_player_game(
    mut commands: Commands,
    mut message_reader: MessageReader<StopSinglePlayerGame>,
    app_role: Res<State<AppRole>>,
    entities_to_despawn: Query<Entity, EntitiesToDespawnQueryFilter>,
) {
    for _ in message_reader.read() {
        if *app_role.get() == AppRole::DedicatedServer {
            info!("Ignoring StopSinglePlayerGame message");
            continue;
        }
        info!("Received StopSinglePlayerGame message!");
        for entity in entities_to_despawn {
            info!("Despawning entity {}", entity);
            commands.entity(entity).despawn();
        }
    }
}
