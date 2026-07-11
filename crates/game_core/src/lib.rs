use netvy::prelude::*;

use avian3d::prelude::*;
use bevy::{
    camera::visibility::RenderLayers, platform::collections::HashMap,
    prelude::*, reflect::TypePath,
};
use bevy_common_assets::json::JsonAssetPlugin;
use serde::{Deserialize, Serialize};
use shared::{
    AppRole, GameConfigServer, GameMap, GameMode, GameStateServer,
    MEDIUM_PLASTIC_MAP_PATH, StartGame, StopGame, TINY_TOWN_MAP_PATH,
    components::Health,
    enemy::components::Enemy,
    game_score::GameScore,
    multiplayer_messages::ClientCommand,
    player::Player,
    world_object::{
        WorldObjectCollectibleKind, WorldObjectCollectibleServerSide,
    },
};

use crate::{
    enemy::{
        EnemyPlugin,
        spawn::{EnemySpawnStrategy, SpawnEnemiesMessage},
    },
    game_flow::{GameFlowPlugin, get_enemy_count_per_wave},
    nav_mesh_pathfinding::NavMeshPathfindingPlugin,
    player::PlayerPlugin,
    world_objects::{WorldObjectsPlugin, components::MapModel},
};

mod enemy;
mod game_flow;
mod nav_mesh_pathfinding;
mod player;
mod world_objects;

// for server binary, this will just be used once, at startup
// a few steps are skipped in case of AppRole::ClientOnly, such as generating the nav mesh or
// spawning the GameScore. maybe i can come up with a better situation

/// Tracks the current state of initializing a game. The main starting point is when a `StartGame`
/// message is read. Then, the first state will be entered. Each state corresponds to one action,
/// and upon finishing each action, it will update to the next action.
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameCoreLoadingState {
    #[default]
    Initial,
    GameScoreFinishedSetup,
    SpawnLocationsLoaded,
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

// TODO: maybe just one universal retry message
#[derive(Message)]
pub struct RetryWaveGameMode;

#[derive(Message)]
pub struct DespawnEnemyMessage {
    pub enemies_to_despawn: Vec<Entity>,
}

#[derive(Message)]
pub struct RequestNewWave;

/// This plugin adds all plugins & systems that need to run on the server, regardless if its for
/// the server binary or the local server that gets started for Singleplayer.
pub struct GameCorePlugin;

impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameCoreLoadingState>();
        app.init_state::<GameStateServer>();

        // any files loaded via the asset server, that end with `spawn_locations.json`, will be
        // parsed into SpawnLocationFile struct, and can then be retrieved via the handle
        app.add_plugins(JsonAssetPlugin::<SpawnLocationFile>::new(&[
            "spawn_locations.json",
        ]));

        app.add_plugins(EnemyPlugin);
        app.add_plugins(NavMeshPathfindingPlugin);
        app.add_plugins(GameFlowPlugin);
        app.add_plugins(WorldObjectsPlugin);
        app.add_plugins(PlayerPlugin);

        app.add_systems(
            Update,
            (
                read_stop_game_message,
                check_world_scene_loaded,
                handle_client_commands,
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

        // app.add_observer(handle_new_connection);
        app.add_observer(check_collider_constructor_hierarchy_ready);

        app.add_systems(
            Update,
            log_updates_to_game_core_loading_state
                .run_if(state_changed::<GameCoreLoadingState>),
        );
    }
}

fn handle_start_game_message(
    mut commands: Commands,
    mut next_server_loading_state: ResMut<NextState<GameCoreLoadingState>>,
    app_role: Res<State<AppRole>>,
    mut start_game_message_reader: MessageReader<StartGame>,
) {
    for message in start_game_message_reader.read() {
        commands.insert_resource(GameConfigServer(message.0));

        info!("Received StartGame message");

        if *app_role.get() != AppRole::ClientOnly {
            commands.spawn((
                GameScore {
                    players: HashMap::new(),
                    enemies: HashMap::new(),
                },
                Name::new("Game Score"),
                ReplicateEntity,
            ));
        }

        // NOTE: theoretically the game score entity is not necessarily already spawned here, but we
        // just do it here as spawning such a simple entity is trivial.
        next_server_loading_state
            .set(GameCoreLoadingState::GameScoreFinishedSetup);
    }
}

// FIXME: So when GameCoreLoadingState is done, why are we still doing stuff? we should rename it.
fn on_game_core_loading_state_done(
    mut commands: Commands,
    mut spawn_enemies: MessageWriter<SpawnEnemiesMessage>,
    enemy_query: Query<Entity, With<Enemy>>,
    game_config_server: Option<Res<GameConfigServer>>,
    app_role: Res<State<AppRole>>,
) {
    if *app_role.get() == AppRole::ClientOnly {
        info!(
            "AppRole is ClientOnly, not doing any actions corresponding to \
             game mode."
        );
        return;
    }

    let Some(game_config_server) = game_config_server else {
        warn!(
            "GameConfigServer doesn't exist, cant execute actions depending \
             on it, like spawning enemies!"
        );
        return;
    };

    let game_mode_server = &game_config_server.0.game_mode;

    info!(
        "GameCoreLoadingState is done, now doing actions corresponding to \
         game mode. Game mode is: {:?}",
        game_mode_server
    );

    match game_mode_server {
        GameMode::Waves => {
            let wave = 1;
            let enemy_count = get_enemy_count_per_wave(wave);
            commands.insert_resource(GameStateWave {
                current_wave: wave,
                enemies_killed: 0,
                enemies_left_from_current_wave: enemy_count,
            });
            spawn_enemies.write(SpawnEnemiesMessage {
                enemy_count,
                spawn_strategy: EnemySpawnStrategy::RandomSelection,
            });
        }
        GameMode::FreeForAll | GameMode::FreeRoam => {
            commands.remove_resource::<GameStateWave>();
            for enemy in enemy_query {
                commands.entity(enemy).despawn();
            }
        }
    };
}

// FIXME: reintroduce
// fn handle_client_respawn_requests(
//     mut commands: Commands,
//     receivers: Query<(&mut NetMessageReader<ClientRespawnRequest>, &PeerId)>,
//     mut player_query: Query<(Entity, &mut Health, &OwnedBy)>,
//     mut server_multi_message_sender: Single<
//         &mut NetMessageWriter<ConfirmRespawn>,
//     >,
//     server: Single<&Server>,
// ) {
//     for (mut message_receiver, peer_id) in receivers {
//         for _ in message_receiver.read() {
//             info!("Received ClientRespawnRequest!");
//             let Some((player_entity, mut player_health, _)) = player_query
//                 .iter_mut()
//                 .find(|(entity, health, owned_by)| owned_by.0 == *peer_id)
//             else {
//                 warn!(
//                     "Read a ClientRespawnRequest but couldn't figure out to \
//                      which player this belongs to"
//                 );
//                 continue;
//             };
//
//             player_health.0 = DEFAULT_HEALTH;
//
//             // TODO: use transform directly
//             // entity_position_server.translation = SPAWN_POINT_MEDIUM_PLASTIC_MAP;
//
//             commands.entity(player_entity).remove::<ColliderDisabled>();
//
//             info!(
//                 "Sending confirm respawn message to client with remote_id: {}",
//                 peer_id.0
//             );
//             // let network_target = &NetworkTarget::Single(peer_id.0);
//
//             let message_sent_result = server_multi_message_sender
//                 .send::<ConfirmRespawn, OrderedReliableChannel>(
//                 &ConfirmRespawn,
//                 &server,
//                 network_target,
//             );
//             match message_sent_result {
//                 Ok(_) => {
//                     info!("Succesfully sent ConfirmRespawn message to client");
//                 }
//                 Err(error) => {
//                     error!(
//                         "Failed to send ConfirmRespawn message to client: {}",
//                         error
//                     );
//                 }
//             }
//         }
//     }
// }

/// ClientCommands exist for the purpose for changing map / game mode on dedicated server, because
/// there, no StartGame message is read, because its not a network message.
fn handle_client_commands(
    query: Query<&mut NetMessageReader<ClientCommand>, With<Server>>,
    mut game_config_server: Option<ResMut<GameConfigServer>>,
    mut game_state_server: ResMut<NextState<GameStateServer>>,
    app_role: Res<State<AppRole>>,
) {
    // the client is not supposed to read client commands. these get handled by the dedicated server
    if *app_role.get() == AppRole::ClientOnly {
        return;
    }

    for mut net_message_reader in query {
        for message in net_message_reader.read() {
            info!("Handling ClientCommand {message:?}");
            let Some(ref mut game_config_server) = game_config_server else {
                warn!(
                    "Received a ClientCommand but GameConfigServer resource \
                     doesnt exist, can't handle ClientCommand."
                );
                return;
            };
            match message {
                ClientCommand::SetGameMode(game_mode) => {
                    game_config_server.0.game_mode = game_mode;
                }
                ClientCommand::SetMap(game_map) => {
                    game_config_server.0.game_map = game_map;
                }
                ClientCommand::SetState(new_game_state_server) => {
                    game_state_server.set(new_game_state_server);
                }
            }
        }
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
    mut game_core_loading_state: ResMut<NextState<GameCoreLoadingState>>,
    mut local_count: Local<usize>,
    game_config: Res<GameConfigServer>,
) {
    *local_count += 1;

    let required_count = match game_config.0.game_map {
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
         GameCoreLoadingState::CollidersSpawned"
    );

    game_core_loading_state.set(GameCoreLoadingState::CollidersSpawned);

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
    world_scene_handle: If<Res<WorldSceneHandle>>,
) {
    for asset_event in asset_event_message_reader.read() {
        if let AssetEvent::LoadedWithDependencies { id } = asset_event
            && *id == world_scene_handle.0.0.id()
        {
            info!(
                "Map fully spawned, updating GameInitializationState -> \
                 GameInitializationState::MapSpawned"
            );
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
    game_config: Res<GameConfigServer>,
) {
    if *app_role.get() == AppRole::DedicatedServer {
        info!("Skipping spawning map, AppRole is DedicatedServer");
        // FIXME: wouldnt this then require to update the loading state?
        return;
    }

    let map_path = match game_config.0.game_map {
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
    info!(
        "\nGameCoreLoadingState UPDATED! Now: {:?}\n",
        *game_core_loading_state.get()
    );
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
        commands.remove_resource::<GameStateWave>();
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
