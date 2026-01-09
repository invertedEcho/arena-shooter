use std::collections::VecDeque;
use std::f32::consts::PI;
use std::time::Duration;

use avian3d::prelude::*;
use bevy::color::palettes;
use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::WgpuSettings;
use bevy_inspector_egui::bevy_egui::{self, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use shared::collider_rules::get_collider_rules_by_map;
use shared::player::{Health, Player, PlayerBundle};
use shared::protocol::{
    ClientUpdatePositionMessage, PlayerPositionServer, ShootRequest,
};
use shared::{
    CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS, SharedPlugin,
};
use shared::{MEDIUM_PLASTIC_MAP_PATH, SERVER_ADDRESS};

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
    let mut app = App::new();

    let run_mode_str = std::env::args().nth(1);
    let run_mode = get_run_mode(run_mode_str.as_ref());
    match run_mode {
        ServerRunMode::Headless => {
            app.add_plugins(
                DefaultPlugins
                    .set(AssetPlugin {
                        file_path: "../../assets".to_string(),
                        ..default()
                    })
                    .set(RenderPlugin {
                        synchronous_pipeline_compilation: true,
                        render_creation:
                            bevy::render::settings::RenderCreation::Automatic(
                                WgpuSettings {
                                    backends: None,
                                    ..default()
                                },
                            ),
                        ..default()
                    }),
            );
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

    app.add_systems(Startup, spawn_map);

    app.run();
}

pub fn setup_server(
    mut commands: Commands,
    server_run_mode: Res<ServerRunMode>,
) {
    let server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig::default()),
            LocalAddr(SERVER_ADDRESS),
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
    mut materials: ResMut<Assets<StandardMaterial>>,
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
        commands.spawn((
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
            PositionHistory {
                samples: VecDeque::from([]),
            },
            Collider::capsule(
                CHARACTER_CAPSULE_RADIUS,
                CHARACTER_CAPSULE_LENGTH,
            ),
            RigidBody::Kinematic,
        ));
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
    players: Query<(Entity, &ControlledBy), With<Player>>,
    mut health_query: Query<&mut Health>,
    spatial_query: SpatialQuery,
) {
    for (mut message_receiver, remote_id) in receivers.iter_mut() {
        for message in message_receiver.receive() {
            let Some(shooter_entity) = players
                .iter()
                .find(|(_, controlled_by)| controlled_by.owner == remote_id)
                .map(|(entity, _)| entity)
            else {
                info!(
                    "Received shootrequest but no corresponding player could \
                     be found"
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

pub fn validate_client_movement() {}

// for now i think i just want to always have a FFA mode on the MediumPlastic map with simple
// scoreboard
pub fn spawn_map(asset_server: Res<AssetServer>, mut commands: Commands) {
    info!("Spawning map on server");

    let map_path = MEDIUM_PLASTIC_MAP_PATH;

    // FIXME: thereotically not needed as server doesnt need light but might be useful for debug
    // purposes to see map on server
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

    let collider_rules =
        get_collider_rules_by_map(&shared::SelectedMapState::MediumPlastic);

    let mut collider_hierarchy = ColliderConstructorHierarchy::new(
        ColliderConstructor::ConvexHullFromMesh,
    );

    for (name, maybe_constructor) in collider_rules {
        match maybe_constructor {
            Some(constructor) => {
                collider_hierarchy = collider_hierarchy
                    .with_constructor_for_name(name, constructor);
            }
            None => {
                collider_hierarchy =
                    collider_hierarchy.without_constructor_for_name(name);
            }
        }
    }

    let world_scene_handle =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(map_path));

    commands.spawn((
        SceneRoot(world_scene_handle),
        Name::new("Medium Plastic Map Scene Root"),
        Visibility::Visible,
        collider_hierarchy,
        RigidBody::Static,
    ));
}

// only needed so we can updated position on server. we may disable this once we have headless
// setup
fn apply_server_position_on_server(
    mut query: Query<(&PlayerPositionServer, &mut Transform), With<Player>>,
) {
    for (server_pos, mut transform) in &mut query {
        transform.translation = server_pos.translation;
    }
}
