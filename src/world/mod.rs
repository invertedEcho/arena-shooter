use crate::world::components::Ground;
use crate::world::systems::setup_world;
use bevy::prelude::*;

pub mod components;
mod systems;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world)
            .register_type::<Ground>();
    }
}
