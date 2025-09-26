use avian3d::prelude::*;
use bevy::{prelude::*, window::PresentMode};
use bevy_hanabi::HanabiPlugin;
use bevy_inspector_egui::{
    bevy_egui::{self, EguiPlugin},
    quick::WorldInspectorPlugin,
};
use bevy_skein::SkeinPlugin;

use crate::{
    common::CommonPlugin, enemy::EnemyPlugin, game_flow::GameFlowPlugin,
    music::MusicPlugin, nav_mesh_pathfinding::NavMeshPathfindingPlugin,
    particles::ParticlesPlugin, player::PlayerPlugin,
    user_interface::UserInterfacePlugin, world::WorldPlugin,
};

mod common;
mod enemy;
mod game_flow;
mod music;
mod nav_mesh_pathfinding;
mod particles;
mod player;
mod user_interface;
mod utils;
mod world;

const GRAVITY: f32 = 9.81;

fn main() {
    let mut app = App::new();

    // bevy-builtin plugins
    app.add_plugins(
        DefaultPlugins.set(bevy::log::LogPlugin {
            // stupid audio library bevy uses which uses info level for debug level messages.. smh
            filter:
                "symphonia_core=off,symphonia_bundle=off,wgpu=error,naga=warn"
                    .to_string(),
            ..default()
        }).set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Fun Shooter".into(),
                    name: Some("fun-shooter".into()),
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            })
        ,
    );
    // app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    // app.add_plugins(LogDiagnosticsPlugin::default());

    // avian (physics)
    app.add_plugins(PhysicsPlugins::default())
        // .add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(Gravity(Vec3::NEG_Y * GRAVITY));

    // skein
    app.add_plugins(SkeinPlugin::default());

    // misc plugins
    app.add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new());

    // hanabi plugins (particles)
    app.add_plugins(HanabiPlugin);

    // own plugins
    app.add_plugins(PlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(GameFlowPlugin)
        .add_plugins(CommonPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(UserInterfacePlugin)
        .add_plugins(ParticlesPlugin)
        .add_plugins(NavMeshPathfindingPlugin)
        .add_plugins(MusicPlugin);

    app.insert_resource(bevy_egui::EguiGlobalSettings {
        auto_create_primary_context: false,
        ..default()
    });

    app.add_systems(Startup, spawn_world_ui_camera);
    app.run();
}

// TODO: i need a a better name for this
#[derive(Component)]
pub struct WorldUiCamera;

fn spawn_world_ui_camera(mut commands: Commands) {
    commands.spawn((
        Camera::default(),
        Camera3d::default(),
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        WorldUiCamera,
    ));
}
