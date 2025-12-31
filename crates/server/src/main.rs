use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use shared::SERVER_ADDRESS;
use shared::player::PlayerBundle;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(ServerPlugins::default());
    app.add_systems(Startup, setup_server);
    app.add_observer(handle_new_client);
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
}

/// A marker component indcating which client this player belongs to
#[derive(Component)]
pub struct ControllerByClient(pub Entity);

pub fn handle_new_client(
    trigger: On<Add, Connected>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    println!("New client! {}", trigger.entity);
    println!("Spawning a player for the new client {}", trigger.entity);

    commands
        .entity(trigger.entity)
        .insert(ReplicationSender::default());

    commands.spawn((
        Name::new("Player"),
        PlayerBundle::default(),
        // Transform::from_translation(player_spawn_location.translation),
        Transform::from_translation(vec3(0.0, 10.0, 0.0)),
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
        ControllerByClient(trigger.entity),
    ));
}
