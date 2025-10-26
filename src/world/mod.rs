use crate::world::components::{Ground, Wall};
use crate::world::messages::{SpawnDebugPointMessage, SpawnMapMessage};
use crate::world::systems::{handle_spawn_debug_points_message, spawn_map};
use bevy::prelude::*;

mod collider_rules;
pub mod components;
pub mod messages;
pub mod resources;
mod systems;

const SMALL_MAP_PATH: &str = "maps/small/main.gltf";
const MEDIUM_MAP_PATH: &str = "maps/medium/scene.gltf#Scene0";

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_spawn_debug_points_message, spawn_map))
            .add_message::<SpawnDebugPointMessage>()
            .add_message::<SpawnMapMessage>()
            .register_type::<Ground>()
            .register_type::<Wall>();
    }
}
