use ::shared::{AppRole, ServerRunMode, SharedPlugin};
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayPlugin, FrameTimeGraphConfig},
    diagnostic::FrameTimeDiagnosticsPlugin,
    input_focus::InputDispatchPlugin,
    prelude::*,
    ui_widgets::UiWidgetsPlugins,
    window::{PresentMode, WindowMode},
};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_hanabi::HanabiPlugin;
use bevy_inspector_egui::{
    bevy_egui::{self, EguiPlugin},
    quick::WorldInspectorPlugin,
};
use bevy_skein::SkeinPlugin;

use crate::{
    audio::AudioPlugin,
    character_controller::CharacterControllerPlugin,
    enemy_visuals::EnemyVisualsPlugin,
    game_flow::GameFlowPlugin,
    game_settings::get_or_create_game_settings,
    gameplay_debug::GameplayDebugPlugin,
    network::NetworkPlugin,
    particles::ParticlesPlugin,
    player::PlayerPlugin,
    shared::{CommonPlugin, systems::apply_render_layers_to_children},
    ui::UserInterfacePlugin,
    world::WorldPlugin,
};

mod audio;
mod auth;
mod character_controller;
mod enemy_visuals;
mod game_flow;
mod game_settings;
mod gameplay_debug;
mod network;
mod particles;
mod player;
mod shared;
mod ui;
mod utils;
mod world;

fn main() {
    let mut app = App::new();

    if cfg!(not(debug_assertions)) {
        app.add_plugins(EmbeddedAssetPlugin {
            mode: bevy_embedded_assets::PluginMode::ReplaceDefault,
        });
    }

    let game_settings = get_or_create_game_settings();

    app.insert_resource(game_settings.clone());

    // app.insert_resource(DirectionalLightShadowMap { size: 4096 });

    let window_mode = if game_settings.graphics.borderless_fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    };

    // bevy-builtin plugins
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Fun Shooter".into(),
                    name: Some("fun-shooter".into()),
                    present_mode: PresentMode::AutoVsync,
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

    // per default, a client is AppRole::ClientOnly. only when player clicks on Singleplayer,
    // AppRole gets set to AppRole::ClientAndServer. once we enter main menu root again, we set it
    // back to ClientOnly
    app.insert_state(AppRole::ClientOnly);

    app.add_plugins(game_core::GameCorePlugin);

    // lightyear plugins
    app.add_plugins(lightyear::prelude::client::ClientPlugins::default());

    app.add_plugins(SharedPlugin);

    app.add_plugins((UiWidgetsPlugins, InputDispatchPlugin));
    app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    app.add_plugins(FpsOverlayPlugin {
        config: bevy::dev_tools::fps_overlay::FpsOverlayConfig {
            text_config: TextFont {
                font_size: 14.,
                ..default()
            },
            enabled: game_settings.graphics.fps_overlay_shown,
            frame_time_graph_config: FrameTimeGraphConfig {
                enabled: false,
                ..default()
            },
            ..default()
        },
    });

    // External plugins
    app.add_plugins(HanabiPlugin); // particles
    app.add_plugins(SkeinPlugin::default()); // use bevy components in blender and have them spawned in the world

    if cfg!(debug_assertions) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(WorldInspectorPlugin::new());
        app.insert_resource(bevy_egui::EguiGlobalSettings {
            auto_create_primary_context: false,
            ..default()
        });
    }

    app.insert_resource(ServerRunMode::Headless);

    app.add_plugins(NetworkPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(GameFlowPlugin)
        .add_plugins(CommonPlugin)
        .add_plugins(UserInterfacePlugin)
        .add_plugins(ParticlesPlugin)
        .add_plugins(CharacterControllerPlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(EnemyVisualsPlugin)
        .add_plugins(WorldPlugin);

    if cfg!(debug_assertions) {
        app.add_plugins(GameplayDebugPlugin);
    }

    // TODO: move elsewhere
    app.add_observer(apply_render_layers_to_children);

    app.run();
}
