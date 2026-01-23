use std::f32::consts::PI;

use ::shared::{
    ServerMode, ServerRunMode, SharedPlugin,
    character_controller::LOCAL_FEET_CHARACTER, enemy::components::Enemy,
    get_auth_backend_socket_addr_client_side,
};
use bevy::{
    dev_tools::fps_overlay::FpsOverlayPlugin,
    diagnostic::FrameTimeDiagnosticsPlugin,
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

use crate::{
    audio::AudioPlugin,
    auth::ConnectTokenRequestTask,
    character_controller::CharacterControllerPlugin,
    client::NetworkPlugin,
    enemy::animate::ENEMY_MODEL_PATH,
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

    app.add_plugins(game_core::ServerPlugin);

    // lightyear plugins
    app.add_plugins(lightyear::prelude::client::ClientPlugins::default());

    app.add_plugins(SharedPlugin);

    app.add_plugins((UiWidgetsPlugins, InputDispatchPlugin));
    app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    app.add_plugins(FpsOverlayPlugin::default());

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
    app.insert_state(ServerMode::LocalServerSinglePlayer);
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

    app.add_systems(Update, spawn_enemy_model_for_new_enemies);

    app.run();
}

pub fn spawn_enemy_model_for_new_enemies(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    enemy_query: Query<Entity, Added<Enemy>>,
) {
    for added_enemy in enemy_query {
        let enemy_model = asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(ENEMY_MODEL_PATH));

        commands.entity(added_enemy).with_child((
            Transform {
                translation: Vec3::new(
                    0.0,
                    // center enemy model -> in blender, feet are at bottom, so in
                    // bevy model feet are at center of collider, 0.0
                    LOCAL_FEET_CHARACTER,
                    0.0,
                ),
                // enemy model needs to be rotated 180 degrees
                rotation: Quat::from_rotation_y(PI),
                ..default()
            },
            SceneRoot(enemy_model),
            Visibility::Visible,
        ));
    }
}
