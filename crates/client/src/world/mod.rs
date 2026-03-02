use crate::world::world_objects::WorldObjectsPlugin;
use bevy::prelude::*;

pub mod components;
pub mod world_objects;

pub struct WorldPlugin;

// FIXME: This should of course not happen on the client, but it will be handled by PR #71
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldObjectsPlugin);
    }
}
