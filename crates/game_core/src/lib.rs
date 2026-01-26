use std::time::Duration;

use avian3d::prelude::*;
use bevy::{color::palettes::css::WHITE, prelude::*};
use lightyear::{
    connection::host::HostClient,
    netcode::NetcodeServer,
    prelude::{
        server::{ClientOf, NetcodeConfig, ServerUdpIo, Start},
        *,
    },
};
use shared::{
    GameModeServer, NETCODE_PROTOCOL_VERSION, SERVER_SOCKET_ADDR_SERVER_SIDE,
    ServerMode, ServerRunMode,
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
    },
    components::Health,
    enemy::components::Enemy,
    get_private_key,
    player::{Player, PlayerBundle},
    protocol::{
        ClientUpdatePositionMessage, EntityPositionServer, ShootRequest,
    },
};

use crate::{
    enemy::{
        EnemyPlugin,
        spawn::{EnemySpawnStrategy, SpawnEnemiesMessage},
    },
    nav_mesh_pathfinding::NavMeshPathfindingPlugin,
};

mod enemy;
mod nav_mesh_pathfinding;

#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum ServerLoadingState {
    #[default]
    Initial,
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
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ServerLoadingState>();

        app.add_plugins(lightyear::prelude::server::ServerPlugins::default());

        app.add_plugins(EnemyPlugin);
        app.add_plugins(NavMeshPathfindingPlugin);

        app.add_systems(Startup, start_server);

        app.add_systems(
            Update,
            (receive_shoot_request, receive_client_update_position),
        );

        app.add_systems(
            OnEnter(ServerLoadingState::Done),
            handle_server_loading_state_done,
        );

        app.add_observer(handle_new_connection);
        app.add_observer(handle_new_client);
    }
}

pub fn start_server(
    mut commands: Commands,
    server_run_mode: Res<ServerRunMode>,
    server_mode: Res<State<ServerMode>>,
) {
    let entity_name = match server_mode.get() {
        ServerMode::LocalServerSinglePlayer => "Local Server for singleplayer",
        ServerMode::RemoteServer => "Server from server Binary",
        ServerMode::None => "Server None",
    };

    let server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig {
                protocol_id: NETCODE_PROTOCOL_VERSION,
                private_key: get_private_key(&server_mode),
                ..default()
            }),
            LocalAddr(SERVER_SOCKET_ADDR_SERVER_SIDE),
            ServerUdpIo::default(),
            Name::new(entity_name),
            GameModeServer::FreeForAll,
        ))
        .id();

    commands.trigger(Start { entity: server });
    if *server_run_mode == ServerRunMode::Headful {
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(10.0, 30.0, 10.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
        ));
        commands.spawn((Node { ..default() }, Text::new("Server")));
    }
}

fn handle_new_connection(
    trigger: On<Add, LinkOf>,
    mut commands: Commands,
    query: Query<(), Without<HostClient>>,
) {
    if query.get(trigger.entity).is_ok() {
        commands.entity(trigger.entity).insert((
            ReplicationSender::new(
                Duration::from_millis(100),
                SendUpdatesMode::SinceLastAck,
                false,
            ),
            Name::from("Client"),
        ));
    }
}

fn handle_new_client(
    trigger: On<Add, Connected>,
    clients_query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    server_mode: Res<State<ServerMode>>,
) {
    if let Ok(remote_id) = clients_query.get(trigger.entity) {
        let client_id = remote_id.0;
        info!(
            "Spawning a player for fully connected Client entity: {} | \
             client_id: {}",
            trigger.entity, client_id
        );

        // NOTE: The replicate component gets inserted into the player entity, but only registered
        // components will be replicated to all other clients
        let player = commands
            .spawn((
                Replicate::to_clients(NetworkTarget::All),
                Name::new("Player"),
                PlayerBundle::default(),
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
                    lifetime: Lifetime::default(),
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
                commands.entity(player).insert((
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
            commands.entity(player).insert(Controlled);
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

fn receive_shoot_request(
    mut receivers: Query<(&mut MessageReceiver<ShootRequest>, Entity)>,
    // TODO: make this more generic, just have a marker component that is like `ShooterEntity` or
    // something?
    player_or_enemies: Query<
        (Entity, &ControlledBy),
        Or<(With<Player>, With<Enemy>)>,
    >,
    mut health_query: Query<&mut Health>,
    spatial_query: SpatialQuery,
) {
    for (mut message_receiver, remote_id) in receivers.iter_mut() {
        for message in message_receiver.receive() {
            let Some(shooter_entity) = player_or_enemies
                .iter()
                .find(|(_, controlled_by)| controlled_by.owner == remote_id)
                .map(|(entity, _)| entity)
            else {
                info!(
                    "Received shootrequest but no corresponding player or \
                     enemy could be found"
                );
                continue;
            };

            let Some(first_hit) = spatial_query.cast_ray(
                message.origin,
                message.direction,
                200.,
                false,
                &SpatialQueryFilter::default()
                    .with_excluded_entities([shooter_entity]),
            ) else {
                continue;
            };

            if let Ok(mut health) = health_query.get_mut(first_hit.entity) {
                health.0 -= 8.0;
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

// TODO: This doesnt work as lightyear inserts Disconnected component by default.
// we probably just need to check for an additional component present in the entity
// fn handle_disconnect(trigger: On<Add, Disconnected>, mut commands: Commands) {
//     info!("Despawning client entity, Disconnected was inserted");
//     commands.entity(trigger.entity).despawn();
// }
