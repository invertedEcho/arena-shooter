use std::sync::{Arc, RwLock};

use bevy::log::LogPlugin;
use bevy::mesh::MeshPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_inspector_egui::bevy_egui::{self, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use game_core::start_server;
use lightyear::utils::collections::HashSet;
use shared::ServerRunMode;
use shared::utils::auth::load_private_key_from_env;
use shared::utils::network::{
    AUTH_BACKEND_ADDRESS_SERVER_SIDE, get_server_socket_addr_client_side,
};
use shared::{AppRole, SharedPlugin};

use crate::auth::start_netcode_authentication_task;
use crate::utils::get_run_mode;

mod auth;
mod utils;

/// This plugin adds all plugins from bevy necessary to start a headless server
struct HeadlessServerPlugin;

impl Plugin for HeadlessServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin {
            file_path: "../../assets".to_string(),
            ..default()
        });
        app.add_plugins(MeshPlugin);
        app.add_plugins((TransformPlugin, bevy::scene::ScenePlugin))
            .init_resource::<Assets<Mesh>>();
        app.add_plugins(LogPlugin::default());
    }
}

/// This plugin adds all plugins from bevy necessary to start a headful server
struct HeadfulServerPlugin;

impl Plugin for HeadfulServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "../../assets".to_string(),
            ..default()
        }));

        app.add_plugins(EguiPlugin::default())
            .add_plugins(WorldInspectorPlugin::new());
        app.insert_resource(bevy_egui::EguiGlobalSettings::default());
    }
}

fn main() {
    dotenvy::dotenv().ok();
    let mut app = App::new();

    let run_mode_str = std::env::args().nth(1);
    let run_mode = get_run_mode(run_mode_str.as_ref());

    app.add_plugins(StatesPlugin);

    app.insert_state(AppRole::DedicatedServer);

    match run_mode {
        ServerRunMode::Headless => {
            app.add_plugins(HeadlessServerPlugin);
            info!("Starting server in headless mode...");
        }
        ServerRunMode::Headful => {
            app.add_plugins(HeadfulServerPlugin);
            info!("Starting server in headful mode...");
        }
    }

    app.add_plugins(game_core::GameCorePlugin);
    app.add_plugins(SharedPlugin);

    app.add_systems(Startup, start_server);

    if run_mode == ServerRunMode::Headful {
        app.add_systems(Startup, spawn_camera_if_headful);
    }

    app.insert_resource(run_mode);

    // authentication
    let client_ids = Arc::new(RwLock::new(HashSet::default()));

    start_netcode_authentication_task(
        // this must be client side because it will be contained in the token that the client
        // receives and uses to connect
        get_server_socket_addr_client_side().expect(
            "Could not resolve game server address. Please make sure you have \
             a working internet connection. Game server may be currently down",
        ),
        AUTH_BACKEND_ADDRESS_SERVER_SIDE,
        client_ids.clone(),
        load_private_key_from_env().unwrap(),
    );

    app.run();
}

fn spawn_camera_if_headful(
    mut commands: Commands,
    server_run_mode: Res<ServerRunMode>,
) {
    if *server_run_mode == ServerRunMode::Headful {
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(10.0, 30.0, 10.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
        ));
        commands.spawn((Node { ..default() }, Text::new("Server")));
    }
}
