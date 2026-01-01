use std::time::Duration;

use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{self, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use shared::SERVER_ADDRESS;
use shared::player::PlayerBundle;
use shared::protocol::ProtocolPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(ServerPlugins::default());
    app.add_plugins(ProtocolPlugin);
    app.add_systems(Startup, setup_server);
    app.add_observer(handle_link_of);
    app.add_observer(handle_new_client);

    if cfg!(debug_assertions) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(WorldInspectorPlugin::new());
        app.insert_resource(bevy_egui::EguiGlobalSettings::default());
    }

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
    commands.spawn((Camera3d::default()));
    commands.spawn((Node { ..default() }, Text::new("Server")));
}

fn handle_link_of(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(ReplicationSender::new(
            Duration::from_millis(100),
            SendUpdatesMode::SinceLastAck,
            false,
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
        info!("FOUND CLIENT_ID: {}", client_id);

        info!("Spawning player");
        commands.spawn((
            Name::new("Player"),
            PlayerBundle::default(),
            // Transform::from_translation(player_spawn_location.translation),
            Transform::from_translation(vec3(0.0, 20.0, 0.0)),
            Visibility::Visible,
            // DebugRender::collider(Color::WHITE),
            // CharacterControllerBundle::default(),
            // DespawnOnExit(AppState::InGame),
            // Mesh3d(meshes.add(Capsule3d::new(
            //     CHARACTER_CAPSULE_RADIUS,
            //     CHARACTER_CAPSULE_LENGTH,
            // ))),
            Mesh3d(meshes.add(Capsule3d::new(0.2, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: WHITE.into(),
                ..Default::default()
            })),
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::All),
            // we add the ControlledBy on the server, with the client entity as the owner of this
            // player, so on the client we can then filter by players that have `Controlled` and
            // those are the players that are actually owned by that client
            ControlledBy {
                owner: trigger.entity,
                lifetime: Lifetime::default(),
            },
        ));
    }
}
