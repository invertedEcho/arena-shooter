use crate::game_flow::states::ClientLoadingState;
use crate::world::systems::{
    on_enter_spawn_map, rotate_and_float_world_objects,
    spawn_visuals_for_world_objects, update_world_object_visibility,
};
use bevy::prelude::*;

pub mod components;
pub mod resources;
mod systems;

// TODO: could have better name for this
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ClientLoadingState::SpawningMap),
            on_enter_spawn_map,
        );
        app.add_systems(
            Update,
            (
                spawn_visuals_for_world_objects,
                rotate_and_float_world_objects,
                update_world_object_visibility,
            ),
        );
    }
}
