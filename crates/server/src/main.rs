use std::collections::VecDeque;
use std::f32::consts::PI;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use avian3d::prelude::*;
use bevy::color::palettes;
use bevy::color::palettes::css::WHITE;
use bevy::log::LogPlugin;
use bevy::mesh::MeshPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{self, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use lightyear::utils::collections::HashSet;
use shared::character_controller::{
    CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
};
use shared::components::Health;
use shared::enemy::components::Enemy;
use shared::player::{Player, PlayerBundle};
use shared::protocol::{
    ClientUpdatePositionMessage, PlayerPositionServer, ShootRequest,
};
use shared::{
    AUTH_BACKEND_ADDRESS_SERVER_SIDE, MEDIUM_PLASTIC_MAP_PATH,
    SERVER_SOCKET_ADDR_SERVER_SIDE, get_server_socket_addr_client_side,
};
use shared::{NETCODE_PROTOCOL_VERSION, SharedPlugin};

use crate::auth::{
    ClientIds, load_private_key, start_netcode_authentication_task,
};
use crate::enemy::EnemyPlugin;

mod auth;
mod enemy;
mod nav_mesh_pathfinding;

#[derive(Resource, PartialEq)]
pub enum ServerRunMode {
    Headless,
    Headful,
}

fn get_run_mode(run_mode_str: Option<&String>) -> ServerRunMode {
    if let Some(run_mode) = run_mode_str {
        if run_mode == "headful" {
            return ServerRunMode::Headful;
        } else if run_mode == "headless" {
            return ServerRunMode::Headless;
        } else {
            warn!(
                "Your given run_mode: {} could not be interpreted. Must \
                 either be 'headless' or 'headful'.",
                run_mode
            );
        }
    }

    ServerRunMode::Headless
}

fn main() {
    dotenvy::dotenv().ok();
    let mut app = App::new();

    let run_mode_str = std::env::args().nth(1);
    let run_mode = get_run_mode(run_mode_str.as_ref());

    match run_mode {
        ServerRunMode::Headless => {
            app.add_plugins(MinimalPlugins);
            app.add_plugins(AssetPlugin {
                file_path: "../../assets".to_string(),
                ..default()
            });
            app.add_plugins(MeshPlugin);
            app.add_plugins((TransformPlugin, bevy::scene::ScenePlugin))
                .init_resource::<Assets<Mesh>>();
            app.add_plugins(LogPlugin::default());
            info!("Running server in headless mode!");
        }
        ServerRunMode::Headful => {
            app.add_plugins(DefaultPlugins.set(AssetPlugin {
                file_path: "../../assets".to_string(),
                ..default()
            }));

            app.add_plugins(EguiPlugin::default())
                .add_plugins(WorldInspectorPlugin::new());
            app.insert_resource(bevy_egui::EguiGlobalSettings::default());
            info!("Running server in headful mode!");
        }
    }
    app.add_plugins(EnemyPlugin);
    app.add_systems(Startup, spawn_map_colliders);

    if run_mode == ServerRunMode::Headful {
        app.add_systems(Startup, spawn_map);
    }

    app.insert_resource(run_mode);

    app.add_plugins(ServerPlugins::default());

    app.add_plugins(SharedPlugin);

    app.add_systems(Startup, setup_server);

    app.add_systems(
        Update,
        (
            apply_server_position_on_server,
            receive_shoot_request,
            receive_client_update_position,
        ),
    );

    app.add_observer(handle_new_connection);
    app.add_observer(handle_new_client);

    // authentication
    let client_ids = Arc::new(RwLock::new(HashSet::default()));
    start_netcode_authentication_task(
        // this must be client side because it will be contained in the token that the client
        // receives and uses to connect
        get_server_socket_addr_client_side(),
        AUTH_BACKEND_ADDRESS_SERVER_SIDE,
        client_ids.clone(),
        load_private_key().expect("Failed to load server private key"),
    );
    app.insert_resource(ClientIds(client_ids));

    app.run();
}

pub fn setup_server(
    mut commands: Commands,
    server_run_mode: Res<ServerRunMode>,
) {
    let server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig {
                protocol_id: NETCODE_PROTOCOL_VERSION,
                private_key: load_private_key()
                    .expect("Failed to load server private key"),
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
                PositionHistory {
                    samples: VecDeque::from([]),
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

#[derive(Component)]
struct PositionHistory {
    samples: VecDeque<(u32, Vec3)>,
}

pub fn receive_client_update_position(
    mut receivers: Query<(
        &mut MessageReceiver<ClientUpdatePositionMessage>,
        Entity,
    )>,
    mut players: Query<
        (&mut PlayerPositionServer, &ControlledBy),
        With<Player>,
    >,
) {
    for (mut message_receiver, remote_id) in receivers.iter_mut() {
        for message in message_receiver.receive() {
            if let Some((mut server_position, _)) = players
                .iter_mut()
                .find(|(_, controlled_by)| controlled_by.owner == remote_id)
            {
                server_position.translation = message.new_translation;
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

// for now i think i just want to always have a FFA mode on the MediumPlastic map with simple
// scoreboard
pub fn spawn_map(asset_server: Res<AssetServer>, mut commands: Commands) {
    info!("Spawning map on server");

    let map_path = MEDIUM_PLASTIC_MAP_PATH;

    commands.spawn((
        Name::new("Map Light"),
        // DespawnOnExit(AppState::InGame),
        DirectionalLight {
            illuminance: 4000.,
            shadows_enabled: true,
            color: palettes::css::ANTIQUE_WHITE.into(),
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 12.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
    ));

    let world_scene_handle =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(map_path));

    commands.spawn((
        SceneRoot(world_scene_handle),
        Name::new("Medium Plastic Map Scene Root"),
        Visibility::Visible,
        RigidBody::Static,
    ));
}

fn spawn_map_colliders(mut commands: Commands) {
    let mut file_buffer = String::from("");
    // FIXME: This will break
    let mut collider_file = File::open(
        "../../assets/maps/medium_plastic/medium_plastic_colliders.json",
    )
    .expect("Can open medium_plastic_colliders.json");
    collider_file.read_to_string(&mut file_buffer).unwrap();

    let colliders: Result<
        Vec<(Collider, GlobalTransform)>,
        serde_json::error::Error,
    > = serde_json::from_str(&file_buffer);

    match colliders {
        Ok(colliders_ok) => {
            info!(
                "Loaded colliders and their transform from json, spawning \
                 them."
            );
            for collider in colliders_ok {
                commands.spawn(collider);
            }
        }
        Err(error) => {
            panic!(
                "Failed to load colliders and their transform from json: {}",
                error
            );
        }
    }
}

fn apply_server_position_on_server(
    mut query: Query<(&PlayerPositionServer, &mut Transform), With<Player>>,
) {
    for (server_pos, mut transform) in &mut query {
        transform.translation = server_pos.translation;
    }
}
