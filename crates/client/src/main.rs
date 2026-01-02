use ::shared::SharedPlugin;
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
use lightyear::prelude::{client::ClientPlugins, *};

use crate::{
    audio::AudioPlugin,
    client::ClientPlugin,
    game_flow::GameFlowPlugin,
    game_settings::get_or_create_game_settings,
    gameplay_debug::GameplayDebugPlugin,
    player::PlayerPlugin,
    shared::{CommonPlugin, systems::apply_render_layers_to_children},
    user_interface::UserInterfacePlugin,
    world::WorldPlugin,
};

// TODO: reintroduce all commented out plugins
mod audio;
// mod character_controller;
mod client;
// mod enemy;
mod game_flow;
mod game_settings;
mod gameplay_debug;
// mod nav_mesh_pathfinding;
// mod particles;
mod player;
mod shared;
mod user_interface;
mod utils;
mod world;

#[derive(Resource)]
pub struct ClientId(pub u64);

fn main() {
    let mut app = App::new();
    let game_settings = get_or_create_game_settings();

    let client_id = std::env::args()
        .nth(1)
        .expect("A client id must be specified");
    app.insert_resource(ClientId(
        client_id.parse().expect("A number must be used"),
    ));

    app.insert_resource(game_settings.clone());

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
                         naga=warn"
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

    // lightyear plugins
    app.add_plugins(ClientPlugins::default());

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

    // FIXME: reintroduce all commented plugins again
    // own plugins
    app.add_plugins(ClientPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(GameFlowPlugin)
        .add_plugins(CommonPlugin)
        // .add_plugins(EnemyPlugin)
        .add_plugins(UserInterfacePlugin)
        // .add_plugins(ParticlesPlugin)
        .add_plugins(AudioPlugin);
    // .add_plugins(NavMeshPathfindingPlugin);

    if cfg!(debug_assertions) {
        app.add_plugins(GameplayDebugPlugin);
    }
    app.add_observer(apply_render_layers_to_children);

    app.run();
}
