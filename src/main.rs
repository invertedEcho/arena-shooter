use avian3d::prelude::*;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{PresentMode, WindowTheme},
};
use bevy_hanabi::HanabiPlugin;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_skein::SkeinPlugin;

use crate::{
    common::CommonPlugin, enemy::EnemyPlugin, game_flow::GameFlowPlugin,
    ground_detection::GroundDetectionPlugin, particles::ParticlesPlugin,
    player::PlayerPlugin, user_interface::UserInterfacePlugin,
    world::WorldPlugin,
};

mod common;
mod enemy;
mod game_flow;
mod ground_detection;
mod particles;
pub mod player;
mod user_interface;
pub mod utils;
mod world;

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
                    title: "fun-shooter".into(),
                    name: Some("fun-shooter".into()),
                    present_mode: PresentMode::AutoVsync,
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
        .add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(Gravity(Vec3::NEG_Y * 9.81));

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
        .add_plugins(GroundDetectionPlugin)
        .add_plugins(CommonPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(UserInterfacePlugin)
        .add_plugins(ParticlesPlugin);
    app.run();
}
