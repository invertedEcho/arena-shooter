use std::num::NonZero;

use avian3d::prelude::*;
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
use bevy_rich_text3d::{
    LoadFonts, Text3d, Text3dPlugin, Text3dStyling, TextAtlas,
    TouchTextMaterial3dPlugin,
};
use bevy_skein::SkeinPlugin;

use crate::{
    character_controller::CharacterControllerPlugin, enemy::EnemyPlugin,
    game_flow::GameFlowPlugin, game_settings::get_or_create_game_settings,
    music::MusicPlugin, nav_mesh_pathfinding::NavMeshPathfindingPlugin,
    particles::ParticlesPlugin, player::PlayerPlugin, shared::CommonPlugin,
    user_interface::UserInterfacePlugin, world::WorldPlugin,
};

mod character_controller;
mod enemy;
mod game_flow;
mod game_settings;
mod music;
mod nav_mesh_pathfinding;
mod particles;
mod player;
mod shared;
mod user_interface;
mod utils;
mod world;

const GRAVITY: f32 = 9.81;

fn main() {
    let game_settings = get_or_create_game_settings();

    let mut app = App::new();
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
            }),
    );
    app.add_plugins((UiWidgetsPlugins, InputDispatchPlugin));
    // app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    // app.add_plugins(LogDiagnosticsPlugin::default());

    // External plugins
    app.add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin)
        .insert_resource(Gravity(Vec3::NEG_Y * GRAVITY));
    app.add_plugins(SkeinPlugin::default());
    app.add_plugins(HanabiPlugin);

    app.add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new());
    app.insert_resource(bevy_egui::EguiGlobalSettings {
        auto_create_primary_context: false,
        ..default()
    });

    // own plugins
    app.add_plugins(PlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(GameFlowPlugin)
        .add_plugins(CommonPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(UserInterfacePlugin)
        .add_plugins(ParticlesPlugin)
        .add_plugins(MusicPlugin)
        .add_plugins(NavMeshPathfindingPlugin)
        .add_plugins(CharacterControllerPlugin);

    app.add_plugins(Text3dPlugin {
        load_system_fonts: true,
        ..Default::default()
    });
    app.add_systems(Startup, spawn_3d_text);

    // Add fonts via the `LoadFonts` resource.
    app.insert_resource(LoadFonts {
        font_paths: vec![
            "assets/fonts/Exo_2/static/Exo2-Regular.ttf".to_owned(),
        ],
        font_directories: vec!["assets/fonts/Exo_2/static".to_owned()],
        ..Default::default()
    });

    app.run();
}

fn spawn_3d_text(
    mut commands: Commands,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let mat = standard_materials.add(StandardMaterial {
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        alpha_mode: AlphaMode::Mask(0.5),
        unlit: true,
        cull_mode: None,
        ..Default::default()
    });
    let test = String::from("test");
    commands.spawn((
        Text3d::new(format!("Hello World from main.rs! {}", test)),
        Text3dStyling {
            size: 64.,
            stroke: NonZero::new(10),
            color: Srgba::new(1., 0., 0., 1.),
            stroke_color: Srgba::BLACK,
            world_scale: Some(Vec2::splat(0.25)),
            layer_offset: 0.001,
            ..Default::default()
        },
        Mesh3d::default(),
        MeshMaterial3d(mat.clone()),
        Transform {
            translation: Vec3::new(1., 1., 4.),
            rotation: Quat::from_axis_angle(Vec3::Y, -30.),
            scale: Vec3::ONE,
        },
    ));
}
