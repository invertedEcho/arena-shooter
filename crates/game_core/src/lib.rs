use std::time::Duration;

use avian3d::prelude::*;
use bevy::{
    camera::visibility::RenderLayers, platform::collections::HashMap,
    prelude::*, reflect::TypePath,
};
use bevy_common_assets::json::JsonAssetPlugin;
use lightyear::{
    netcode::NetcodeServer,
    prelude::{
        server::{NetcodeConfig, ServerUdpIo, Start},
        *,
    },
};
use serde::{Deserialize, Serialize};
use shared::{
    AppRole, DEFAULT_HEALTH, GameMap, GameModeServer, GameStateServer,
    MEDIUM_PLASTIC_MAP_PATH, SPAWN_POINT_MEDIUM_PLASTIC_MAP, StartGame,
    StopGame, TINY_TOWN_MAP_PATH,
    components::{EntityPositionServer, Health},
    enemy::components::Enemy,
    game_score::GameScore,
    multiplayer_messages::{
        ChangeGameServerStateRequest, ClientRespawnRequest,
        ClientUpdatePositionMessage, ConfirmRespawn,
    },
    player::Player,
    protocol::OrderedReliableChannel,
    utils::{
        auth::{LOCAL_SERVER_PRIVATE_KEY, load_private_key_from_env},
        network::{
            NETCODE_PROTOCOL_VERSION, SERVER_SOCKET_ADDR_DEDICATED_SERVER,
            SERVER_SOCKET_ADDR_SINGLEPLAYER,
        },
    },
    world_object::{
        WorldObjectCollectibleKind, WorldObjectCollectibleServerSide,
    },
};

use crate::{
    enemy::{
        EnemyPlugin,
        spawn::{EnemySpawnStrategy, SpawnEnemiesMessage},
    },
    game_flow::GameFlowPlugin,
    nav_mesh_pathfinding::NavMeshPathfindingPlugin,
    player::PlayerPlugin,
    world_objects::{MapPlugin, components::MapModel},
};

mod enemy;
mod game_flow;
mod nav_mesh_pathfinding;
mod player;
mod world_objects;

// on client, the state gets reset to Initial when we exit to main menu, as everything gets
// despawned.
// for server binary, this will just be used once, at startup
// a few steps are skipped in case of AppRole::ClientOnly, such as generating the nav mesh or
// spawning the GameScore. maybe i can come up with a better situation
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameCoreLoadingState {
    #[default]
    Initial,
    GameScoreFinishedSetup,
    // FIXME: MapSpawned is currently never entered again after the first game
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
        app.init_state::<GameMap>();
        app.init_state::<GameModeServer>();

        app.add_plugins(JsonAssetPlugin::<SpawnLocationFile>::new(&[
            "spawn_location.json",
        ]));

        app.add_plugins(lightyear::prelude::server::ServerPlugins::default());

        app.add_plugins(EnemyPlugin);
        app.add_plugins(NavMeshPathfindingPlugin);
        app.add_plugins(GameFlowPlugin);
        app.add_plugins(MapPlugin);
        app.add_plugins(PlayerPlugin);

        app.add_systems(
            Update,
            (
                receive_and_apply_client_update_position
                    .run_if(in_state(AppRole::DedicatedServer)),
                handle_client_respawn_requests,
                handle_game_server_state_update_request,
                read_stop_game_message,
                check_world_scene_loaded,
            ),
        );
        app.add_systems(
            Update,
            (kill_players_below_death_zone)
                .run_if(not(in_state(AppRole::ClientOnly))),
        );

        app.add_systems(
            OnEnter(GameCoreLoadingState::Done),
            on_game_core_loading_state_done,
        );

        app.add_systems(Update, handle_start_game_message);

        app.add_systems(
            OnEnter(GameCoreLoadingState::GameScoreFinishedSetup),
            spawn_map,
        );

        app.add_observer(handle_new_connection);
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
            DespawnOnExit(AppRole::ClientAndServer),
        ))
        .id();

    commands.trigger(Start { entity: server });
}

fn handle_start_game_message(
    mut commands: Commands,
    mut next_server_loading_state: ResMut<NextState<GameCoreLoadingState>>,
    app_role: Res<State<AppRole>>,
    mut start_game_message_reader: MessageReader<StartGame>,
    mut next_current_map: ResMut<NextState<GameMap>>,
    mut game_mode_server: ResMut<NextState<GameModeServer>>,
) {
    for message in start_game_message_reader.read() {
        info!("Received StartGame message");
        next_current_map.set(message.map.clone());

        if *app_role.get() != AppRole::ClientOnly {
            game_mode_server.set(message.game_mode.clone());
            info!(
                "Updated GameModeServer to {:?}, read from StartGame message.",
                message.game_mode
            );
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
        }

        // NOTE: theoretically the game score entity is not necessarily already spawned here, but we
        // just do it here as spawning such a simple entity is trivial.
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

/// This systems receives a message from clients, that their position has changed.
/// The server will then apply it to the `PlayerPositionServer` component, which then gets
/// replicated to all clients. All clients receive the updates from `PlayerPositionServer`, and
/// update the Transform locally.
fn receive_and_apply_client_update_position(
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

fn on_game_core_loading_state_done(
    mut commands: Commands,
    game_mode_server: Res<State<GameModeServer>>,
    mut spawn_enemies: MessageWriter<SpawnEnemiesMessage>,
    enemy_query: Query<Entity, With<Enemy>>,
    app_role: Res<State<AppRole>>,
) {
    if *app_role.get() == AppRole::ClientOnly {
        info!(
            "Not doing actions depending on GameModeServer, this is ClientOnly"
        );
        return;
    }
    // TODO: This would mean GameCore is not fully done? We still spawn enemies, so theoretically
    // GameCoreLoadingState should not be done, e.g. we should add another state
    info!(
        "GameCoreLoadingState is done, now doing actions corresponding to \
         game mode. Game mode is: {:?}",
        *game_mode_server
    );

    match *game_mode_server.get() {
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
                            error!(
                                "Failed to find controlling player: {}",
                                error
                            );
                        }
                    }
                }
                None => {
                    error!(
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

const COLLIDER_CONSTRUCTOR_COUNT_MEDIUM_PLASTIC: usize = 2;

// On tiny town we also have bunch of CollderConstructor, but they are all of type Cuboid, so very
// easy to spawn
const COLLIDER_CONSTRUCTOR_COUNT_TINY_TOWN: usize = 2;

fn check_collider_constructor_hierarchy_ready(
    _trigger: On<ColliderConstructorHierarchyReady>,
    mut next_server_loading_state: ResMut<NextState<GameCoreLoadingState>>,
    mut local_count: Local<usize>,
    current_map: Res<State<GameMap>>,
) {
    *local_count += 1;

    let required_count = match current_map.get() {
        GameMap::MediumPlastic => COLLIDER_CONSTRUCTOR_COUNT_MEDIUM_PLASTIC,
        GameMap::TinyTown => COLLIDER_CONSTRUCTOR_COUNT_TINY_TOWN,
    };

    // Only after all ColliderConstructorHierarchy are ready, we update
    // the GameCoreLoadingState to CollidersSpawned
    if *local_count != required_count {
        return;
    }

    info!(
        "All ColliderConstructorHierarchies are ready, setting \
         ServerLoadingState::CollidersSpawned"
    );

    next_server_loading_state.set(GameCoreLoadingState::CollidersSpawned);

    // Reset back to zero to prepare for next GameStart
    *local_count = 0;
}

/// We store the world scene handle as we listen for any AssetEvents::Scene to be
/// `LoadedWithDependencies`. but this of course gets triggered for any scenes that get spawned,
/// like enemy models, so we need to compare our WorldSceneHandle that we insert when we spawn the
/// map with the one that we get from the LoadedWithDependencies message/event.
#[derive(Resource, Debug)]
pub struct WorldSceneHandle(pub Handle<Scene>);

// FIXME: this detection logic doesnt work on second time
fn check_world_scene_loaded(
    mut asset_event_message_reader: MessageReader<AssetEvent<Scene>>,
    mut next_game_core_loading_state: ResMut<NextState<GameCoreLoadingState>>,
    maybe_world_scene_handle: Option<Res<WorldSceneHandle>>,
) {
    for asset_event in asset_event_message_reader.read() {
        if let AssetEvent::LoadedWithDependencies { id } = asset_event
            && let Some(ref world_scene_handle) = maybe_world_scene_handle
            && *id == world_scene_handle.0.id()
        {
            info!("Map fully spawned");
            next_game_core_loading_state.set(GameCoreLoadingState::MapSpawned);
        }
    }
}

#[derive(Component)]
struct GameMapLight;

fn spawn_map(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    app_role: Res<State<AppRole>>,
    current_map: Res<State<GameMap>>,
) {
    // FIXME: not spawning the map actually has the problem that the MedkitSpawnLocations and
    // AmmunitionPackSpawnLocations are never spawned. so dedicated server will never have those
    // we need another way of storing information on where to spawn medkits, etc
    if *app_role.get() == AppRole::DedicatedServer {
        info!("Skipping spawning map, AppRole is DedicatedServer");
        return;
    }

    let map_path = match current_map.get() {
        GameMap::MediumPlastic => MEDIUM_PLASTIC_MAP_PATH,
        GameMap::TinyTown => TINY_TOWN_MAP_PATH,
    };

    info!(
        "Entered GameCoreLoadingState::GameScoreFinishedSetup! Spawning the \
         game map"
    );
    commands.spawn((
        DirectionalLight {
            illuminance: 6000.,
            shadows_enabled: true,
            ..default()
        },
        GameMapLight,
        Transform::default().looking_at(Vec3::new(-1.0, -3.0, -2.0), Vec3::Y),
        // TODO: should be constants
        RenderLayers::from_layers(&[0, 1]),
    ));

    let world_scene_handle =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(map_path));

    commands.insert_resource(WorldSceneHandle(world_scene_handle.clone()));

    commands.spawn((
        SceneRoot(world_scene_handle),
        Name::new("Scene Root (Map)"),
        Visibility::Visible,
        RigidBody::Static,
        MapModel,
    ));
}

fn log_updates_to_game_core_loading_state(
    game_core_loading_state: Res<State<GameCoreLoadingState>>,
) {
    println!();
    info!(
        "GameCoreLoadingState UPDATED! Now: {:?}",
        *game_core_loading_state.get()
    );
    println!();
}

type EntitiesToDespawnQueryFilter = Or<(
    With<GameMapLight>,
    With<Enemy>,
    With<Server>,
    With<Client>,
    With<GameScore>,
    With<MapModel>,
    With<WorldObjectCollectibleServerSide>,
)>;

fn read_stop_game_message(
    mut commands: Commands,
    mut message_reader: MessageReader<StopGame>,
    app_role: Res<State<AppRole>>,
    entities_to_despawn: Query<Entity, EntitiesToDespawnQueryFilter>,
) {
    for _ in message_reader.read() {
        if *app_role.get() == AppRole::DedicatedServer {
            info!("Ignoring StopGame message");
            continue;
        }
        info!("Received StopGame message!");
        for entity in entities_to_despawn {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Asset, TypePath)]
pub struct SpawnLocationFile {
    positions: Vec<SpawnLocation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpawnLocation {
    kind: WorldObjectCollectibleKind,
    position: Vec3,
}
