use crate::game_flow::states::LoadingGameState;
use crate::world::systems::on_enter_spawn_map;
use crate::world::world_objects::WorldObjectsPlugin;
use bevy::prelude::*;

pub mod resources;
mod systems;
pub mod world_objects;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldObjectsPlugin).add_systems(
            OnEnter(LoadingGameState::SpawningMap),
            on_enter_spawn_map,
        );
    }
}
