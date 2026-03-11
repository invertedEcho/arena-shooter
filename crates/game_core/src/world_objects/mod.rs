use bevy::prelude::*;
use shared::GameMap;

use crate::{
    GameCoreLoadingState,
    world_objects::systems::{
        activate_world_objects_over_time,
        detect_collision_world_object_with_player, load_spawn_locations,
        spawn_world_objects, tick_respawn_timer_world_objects,
    },
};

pub mod components;
mod systems;

const DEFAULT_HEALTH_TO_GIVE_MEDKIT: f32 = 25.0;

pub struct WorldObjectsPlugin;

impl Plugin for WorldObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            load_spawn_locations.run_if(state_changed::<GameMap>),
        );
        app.add_systems(
            OnEnter(GameCoreLoadingState::GameScoreFinishedSetup),
            spawn_world_objects,
        );
        app.add_systems(
            Update,
            (
                detect_collision_world_object_with_player,
                activate_world_objects_over_time,
                tick_respawn_timer_world_objects,
            ),
        );
    }
}
