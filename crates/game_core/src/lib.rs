use std::time::Duration;

use avian3d::prelude::*;
use bevy::{color::palettes::css::WHITE, prelude::*};
use lightyear::{
    netcode::NetcodeServer,
    prelude::{
        server::{ClientOf, NetcodeConfig, ServerUdpIo, Start},
        *,
    },
};
use shared::{
    NETCODE_PROTOCOL_VERSION, SERVER_SOCKET_ADDR_SERVER_SIDE, ServerMode,
    ServerRunMode,
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
    },
    components::Health,
    enemy::components::Enemy,
    get_private_key,
    player::{Player, PlayerBundle},
    protocol::{
        ClientUpdatePositionMessage, PlayerPositionServer, ShootRequest,
    },
};

use crate::{
    enemy::EnemyPlugin, nav_mesh_pathfinding::NavMeshPathfindingPlugin,
};

mod enemy;
mod nav_mesh_pathfinding;

/// This plugin adds all plugins & systems that need to run on the server, regardless if its for
/// the server binary or the local server that gets started for Singleplayer.
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(lightyear::prelude::server::ServerPlugins::default());

        app.add_plugins(EnemyPlugin);
        app.add_plugins(NavMeshPathfindingPlugin);

        app.add_systems(Startup, start_server);

        app.add_systems(
            Update,
            (receive_shoot_request, receive_client_update_position),
        );

        app.add_observer(handle_new_connection);
        app.add_observer(handle_new_client);
    }
}

// TODO: add Resource to get/set LocalAddr for NetcodeServer
// and also private key
pub fn start_server(
    mut commands: Commands,
    server_run_mode: Res<ServerRunMode>,
    server_mode: Res<ServerMode>,
) {
    let server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig {
                protocol_id: NETCODE_PROTOCOL_VERSION,
                private_key: get_private_key(&server_mode),
                ..default()
            }),
            LocalAddr(SERVER_SOCKET_ADDR_SERVER_SIDE),
            ServerUdpIo::default(),
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

fn handle_new_connection(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands.entity(trigger.entity).insert((
        ReplicationSender::new(
            Duration::from_millis(100),
            SendUpdatesMode::SinceLastAck,
            false,
        ),
        Name::from("Client"),
    ));
}

fn handle_new_client(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if let Ok(remote_id) = query.get(trigger.entity) {
        let client_id = remote_id.0;
        info!(
            "Spawning player for fully connected Client entity: {} | \
             client_id: {}",
            trigger.entity, client_id
        );

        // NOTE: The replicate component gets inserted into the player entity, but only registered
        // components will be replicated to all other clients
        let client = commands
            .spawn((
                Replicate::to_clients(NetworkTarget::All),
                Name::new("Player"),
                PlayerBundle::default(),
                // TODO: think we could override replication behaviour for this component and only
                // replicate to all other clients than the current client
                PlayerPositionServer {
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

        // on headless setup, materials doesnt exist
        if let Some(mut materials) = materials {
            commands.entity(client).insert((
                // thereotically this isnt needed, as each client inserts the mesh with material when a
                // new player is added, but for now we keep it so we can see what happens on the server.
                // meshes are not replicated
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
}

/// This systems receives a message from clients, that their position has changed.
/// The server will then apply it to the `PlayerPositionServer` component, which then gets
/// replicated to all clients. All clients receive the updates from `PlayerPositionServer`, and
/// update the Transform locally.
pub fn receive_client_update_position(
    mut receivers: Query<(
        &mut MessageReceiver<ClientUpdatePositionMessage>,
        Entity,
    )>,
    mut players: Query<
        (&mut PlayerPositionServer, &mut Transform, &ControlledBy),
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

pub fn receive_shoot_request(
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
