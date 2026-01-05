use crate::world::messages::SpawnMapMessage;
use crate::world::systems::handle_spawn_map_message;
use crate::world::world_objects::WorldObjectsPlugin;
use bevy::prelude::*;

pub mod collider_rules;
pub mod messages;
pub mod resources;
mod systems;
pub mod world_objects;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldObjectsPlugin)
            .add_message::<SpawnMapMessage>()
            .add_systems(Update, handle_spawn_map_message);
    }
}
