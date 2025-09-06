use bevy::prelude::*;

use crate::debug_hud::systems::spawn_debug_hud;

pub mod systems;

#[derive(Component)]
pub struct RootNode;

pub struct DebugHudPlugin;

impl Plugin for DebugHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_debug_hud);
    }
}
