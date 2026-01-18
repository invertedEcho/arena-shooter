use ::shared::{
    ServerMode, ServerRunMode, SharedPlugin,
    get_auth_backend_socket_addr_client_side,
};
use bevy::{
    input_focus::InputDispatchPlugin,
    prelude::*,
    ui_widgets::UiWidgetsPlugins,
    window::{PresentMode, WindowMode},
};
use bevy_hanabi::HanabiPlugin;
use bevy_inspector_egui::{
    bevy_egui::{self, EguiPlugin},
    quick::WorldInspectorPlugin,
};
use lightyear::connection::host::HostPlugin;

use crate::{
    audio::AudioPlugin,
    auth::ConnectTokenRequestTask,
    character_controller::CharacterControllerPlugin,
    client::NetworkPlugin,
    game_flow::GameFlowPlugin,
    game_settings::get_or_create_game_settings,
    gameplay_debug::GameplayDebugPlugin,
    particles::ParticlesPlugin,
    player::PlayerPlugin,
    shared::{CommonPlugin, systems::apply_render_layers_to_children},
    user_interface::UserInterfacePlugin,
    world::WorldPlugin,
};

use enemy::animate::AnimateEnemyPlugin;

mod audio;
mod auth;
mod character_controller;
mod client;
mod enemy;
mod game_flow;
mod game_settings;
mod gameplay_debug;
mod particles;
mod player;
mod shared;
mod user_interface;
mod world;

fn main() {
    let mut app = App::new();
    let game_settings = get_or_create_game_settings();

    app.insert_resource(game_settings.clone());

    app.insert_resource(ConnectTokenRequestTask {
        auth_backend_addr: get_auth_backend_socket_addr_client_side(),
        task: None,
    });

    let window_mode = if game_settings.fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    };

    // bevy-builtin plugins
    app.add_plugins(
        DefaultPlugins
            .set(bevy::log::LogPlugin {
                // stupid audio library bevy uses which uses info level for debug level messages.. smh
                filter: "symphonia_core=off,symphonia_bundle=off,wgpu=error,\
                         naga=warn,lightyear=debug"
                    .to_string(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Fun Shooter".into(),
                    name: Some("fun-shooter".into()),
                    present_mode: PresentMode::AutoNoVsync,
                    mode: window_mode,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: "../../assets".to_string(),
                ..default()
            }),
    );

    app.add_plugins(game_core::ServerPlugin);

    // lightyear plugins
    app.add_plugins(lightyear::prelude::client::ClientPlugins::default());

    app.add_plugins(SharedPlugin);

    app.add_plugins((UiWidgetsPlugins, InputDispatchPlugin));
    // app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    // app.add_plugins(LogDiagnosticsPlugin::default());

    // External plugins
    app.add_plugins(HanabiPlugin);

    if cfg!(debug_assertions) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(WorldInspectorPlugin::new());
        app.insert_resource(bevy_egui::EguiGlobalSettings {
            auto_create_primary_context: false,
            ..default()
        });
    }

    app.insert_resource(ServerRunMode::Headless);
    app.insert_resource(ServerMode::LocalServerSinglePlayer);
    app.add_plugins(NetworkPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(GameFlowPlugin)
        .add_plugins(CommonPlugin)
        .add_plugins(UserInterfacePlugin)
        .add_plugins(ParticlesPlugin)
        .add_plugins(CharacterControllerPlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(AnimateEnemyPlugin);

    if cfg!(debug_assertions) {
        app.add_plugins(GameplayDebugPlugin);
    }

    app.add_observer(apply_render_layers_to_children);

    app.run();
}
