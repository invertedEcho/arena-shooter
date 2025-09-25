use crate::world::components::{Ground, Map, Wall};
use crate::world::systems::{
    detect_bullet_collision_with_wall_and_grounds, setup_world,
};
use bevy::prelude::*;

pub mod components;
mod systems;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world)
            .add_systems(
                Update,
                (detect_bullet_collision_with_wall_and_grounds,),
            )
            .register_type::<Ground>()
            .register_type::<Map>()
            .register_type::<Wall>();
    }
}
