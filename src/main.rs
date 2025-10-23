use avian3d::prelude::*;
use bevy::{prelude::*, window::PresentMode};
use bevy_hanabi::HanabiPlugin;
use bevy_inspector_egui::{
    bevy_egui::{self, EguiPlugin},
    quick::WorldInspectorPlugin,
};
use bevy_rich_text3d::Text3dPlugin;
use bevy_skein::SkeinPlugin;

use crate::{
    enemy::EnemyPlugin, game_flow::GameFlowPlugin, music::MusicPlugin,
    nav_mesh_pathfinding::NavMeshPathfindingPlugin, particles::ParticlesPlugin,
    player::PlayerPlugin, shared::CommonPlugin,
    user_interface::UserInterfacePlugin, world::WorldPlugin,
};

mod enemy;
mod game_flow;
mod music;
mod nav_mesh_pathfinding;
mod particles;
mod player;
mod shared;
mod user_interface;
mod utils;
mod world;

// TODO: convert all static things into one mesh for a single mesh instead of single ones ->
// probably trimesh_from_mesh

const GRAVITY: f32 = 9.81;

fn main() {
    let mut app = App::new();

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
                    ..default()
                }),
                ..default()
            }),
    );
    // app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    // app.add_plugins(LogDiagnosticsPlugin::default());

    // External plugins
    app.add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(Gravity(Vec3::NEG_Y * GRAVITY))
        .insert_gizmo_config(
            PhysicsGizmos {
                // disable axis gizmos as player camera is exactly at center of collider
                axis_lengths: Some(Vec3::splat(0.0)),
                ..default()
            },
            GizmoConfig::default(),
        );
    app.add_plugins(SkeinPlugin::default());
    app.add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new());
    app.add_plugins(HanabiPlugin);
    app.add_plugins(Text3dPlugin::default());

    // own plugins
    app.add_plugins(PlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(GameFlowPlugin)
        .add_plugins(CommonPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(UserInterfacePlugin)
        .add_plugins(ParticlesPlugin)
        .add_plugins(MusicPlugin)
        .add_plugins(NavMeshPathfindingPlugin);

    app.insert_resource(bevy_egui::EguiGlobalSettings {
        auto_create_primary_context: false,
        ..default()
    });

    app.run();
}
