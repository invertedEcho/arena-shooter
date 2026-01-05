use std::time::Duration;

use avian3d::math::PI;
use avian3d::prelude::*;
use bevy::color::palettes;
use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{self, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use lightyear::prelude::client::InputDelayConfig;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use shared::SharedPlugin;
use shared::collider_rules::get_collider_rules_by_map;
use shared::player::{Player, PlayerBundle};
use shared::protocol::ClientUpdatePositionMessage;
use shared::{MEDIUM_PLASTIC_MAP_PATH, SERVER_ADDRESS};

fn main() {
    let mut app = App::new();
    // TODO: use mimimalplugins
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        file_path: "../../assets".to_string(),
        ..default()
    }));
    app.add_plugins(ServerPlugins::default());

    app.add_plugins(SharedPlugin);

    app.add_systems(Startup, setup_server);

    app.add_systems(Update, receive_client_update_position);

    app.add_observer(handle_new_connection);
    app.add_observer(handle_new_client);

    if cfg!(debug_assertions) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(WorldInspectorPlugin::new());
        app.insert_resource(bevy_egui::EguiGlobalSettings::default());
    }

    app.add_systems(Startup, spawn_map);

    app.run();
}

pub fn setup_server(mut commands: Commands) {
    let server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig::default()),
            LocalAddr(SERVER_ADDRESS),
            ServerUdpIo::default(),
        ))
        .id();

    commands.trigger(Start { entity: server });
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(10.0, 30.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((Node { ..default() }, Text::new("Server")));
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

        info!("Spawning player");
        commands.spawn((
            Name::new("Player"),
            PlayerBundle::default(),
            Transform::from_translation(vec3(0.0, 20.0, 0.0)),
            Visibility::Visible,
            // DebugRender::collider(Color::WHITE),
            // PredictionTarget::to_clients(NetworkTarget::All),
            Replicate::to_clients(NetworkTarget::All),
            // we add the ControlledBy on the server, with the client entity as the owner of this
            // player, so on the client we can then filter by players that have `Controlled` and
            // those are the players that are actually owned by that client
            ControlledBy {
                owner: trigger.entity,
                lifetime: Lifetime::default(),
            },
            Mesh3d(meshes.add(Capsule3d::new(0.2, 1.3))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: WHITE.into(),
                ..Default::default()
            })),
        ));
    }
}

pub fn receive_client_update_position(
    receivers: Query<(
        &mut MessageReceiver<ClientUpdatePositionMessage>,
        Entity,
    )>,
    mut players: Query<(&mut Transform, &ControlledBy), With<Player>>,
) {
    for (mut message_receiver, remote_id) in receivers {
        for message in message_receiver.receive() {
            if let Some((mut transform, _)) = players
                .iter_mut()
                .find(|(_, controlled_by)| controlled_by.owner == remote_id)
            {
                info!(
                    "Found corresponding player from \
                     ClientUpdatePositionMessage message!, current transform: \
                     {:?}",
                    transform
                );
                info!("Updating transform of player on server");
                *transform = message.new_transform;
            } else {
                warn!(
                    "Received a ClientUpdatePositionMessage but couldnt find \
                     the corresponding Player entity on the server"
                );
            }
        }
    }
}

pub fn validate_client_movement() {}

// for now i think i just want to always have a FFA mode on the MediumPlastic map with simple
// scoreboard
pub fn spawn_map(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    // selected_map_state: Res<State<SelectedMapState>>,
) {
    // let selected_map = selected_map_state.get();
    let map_path = MEDIUM_PLASTIC_MAP_PATH;

    info!("Spawning map on server");
    // info!(
    //     "Entered LoadingGameSubState::SpawningMap, spawning map {:?} with \
    //      path {:?}",
    //     selected_map, map_path
    // );

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

    let world_scene_handle =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(map_path));

    // commands.insert_resource(WorldSceneHandle(world_scene_handle.clone()));

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

    commands.spawn((
        // DespawnOnExit(AppState::InGame),
        SceneRoot(world_scene_handle),
        collider_hierarchy,
        Name::new("World Scene Root"),
        Visibility::Visible,
        RigidBody::Static,
    ));
}
