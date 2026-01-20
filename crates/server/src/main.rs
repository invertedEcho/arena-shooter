use std::f32::consts::PI;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, RwLock};

use avian3d::prelude::*;
use bevy::color::palettes;
use bevy::log::LogPlugin;
use bevy::mesh::MeshPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{self, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use lightyear::utils::collections::HashSet;
use shared::{
    AUTH_BACKEND_ADDRESS_SERVER_SIDE, MEDIUM_PLASTIC_MAP_PATH, ServerMode,
    ServerRunMode, get_server_socket_addr_client_side,
};
use shared::{SharedPlugin, get_private_key};

use crate::auth::{ClientIds, start_netcode_authentication_task};

mod auth;

fn get_run_mode(run_mode_str: Option<&String>) -> ServerRunMode {
    if let Some(run_mode) = run_mode_str {
        if run_mode == "headful" {
            return ServerRunMode::Headful;
        } else if run_mode == "headless" {
            return ServerRunMode::Headless;
        } else {
            warn!(
                "Your given run_mode: {} could not be interpreted. Must \
                 either be 'headless' or 'headful'. Defaulting to headless.",
                run_mode
            );
        }
    }

    ServerRunMode::Headless
}

/// This plugin adds all plugins from bevy necessary to start a headless server
pub struct HeadlessServerPlugin;

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

/// This plugin adds all plugins from bevy necessary to start a headless server
pub struct HeadfulServerPlugin;

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

/// This plugin adds plugins & systems thats only relevant if the server is the server binary
/// itself.
pub struct MultiPlayerServerOnlyPlugin;

impl Plugin for MultiPlayerServerOnlyPlugin {
    fn build(&self, app: &mut App) {
        // if we would add sharedplugin in ServerPlugin, it would already be added by client
        app.add_plugins(SharedPlugin);
    }
}

fn main() {
    dotenvy::dotenv().ok();
    let mut app = App::new();

    let run_mode_str = std::env::args().nth(1);
    let run_mode = get_run_mode(run_mode_str.as_ref());

    match run_mode {
        ServerRunMode::Headless => {
            app.add_plugins(HeadlessServerPlugin);
            info!("Running server in headless mode!");
        }
        ServerRunMode::Headful => {
            app.add_plugins(HeadfulServerPlugin);
            info!("Running server in headful mode!");
        }
    }
    app.add_systems(Startup, spawn_map_colliders);

    if run_mode == ServerRunMode::Headful {
        app.add_systems(Startup, spawn_map);
    }

    app.insert_resource(run_mode);

    // authentication
    let client_ids = Arc::new(RwLock::new(HashSet::default()));
    start_netcode_authentication_task(
        // this must be client side because it will be contained in the token that the client
        // receives and uses to connect
        get_server_socket_addr_client_side(),
        AUTH_BACKEND_ADDRESS_SERVER_SIDE,
        client_ids.clone(),
        get_private_key(&ServerMode::RemoteServer),
    );
    app.insert_resource(ClientIds(client_ids));

    app.run();
}

// for now i think i just want to always have a FFA mode on the MediumPlastic map with simple
// scoreboard
pub fn spawn_map(asset_server: Res<AssetServer>, mut commands: Commands) {
    info!("Spawning map on server");

    let map_path = MEDIUM_PLASTIC_MAP_PATH;

    commands.spawn((
        Name::new("Map Light"),
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
