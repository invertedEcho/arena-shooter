use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_skein::SkeinPlugin;

use crate::{
    common::CommonPlugin, debug_hud::DebugHudPlugin, enemy::EnemyPlugin,
    game_flow::GameFlowPlugin, ground_detection::GroundDetectionPlugin,
    player::PlayerPlugin, world::WorldPlugin,
};

mod common;
mod debug_hud;
mod enemy;
mod game_flow;
mod ground_detection;
pub mod player;
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
        }),
    );

    // avian (physics)
    app.add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(Gravity(Vec3::NEG_Y * 9.81));

    // own plugins
    app.add_plugins(PlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(GameFlowPlugin)
        .add_plugins(DebugHudPlugin)
        .add_plugins(GroundDetectionPlugin)
        .add_plugins(CommonPlugin)
        .add_plugins(EnemyPlugin);

    // skein
    app.add_plugins(SkeinPlugin::default());

    // misc plugins
    // app.add_plugins(EditorPlugin::default());

    app.run();
}
