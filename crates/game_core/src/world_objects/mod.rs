use bevy::prelude::*;

use crate::{
    GameCoreLoadingState,
    world_objects::systems::{
        activate_world_objects_over_time,
        detect_collision_world_object_with_player, spawn_world_objects,
        tick_respawn_timer_world_objects,
    },
};

pub mod components;
mod systems;

const DEFAULT_HEALTH_TO_GIVE_MEDKIT: f32 = 25.0;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        // for some reason, spawn_world_objects makes client crash when this is set to
        // OnEnter(GameCoreLoadingState::GameScoreFinishedSetup)
        app.add_systems(
            OnEnter(GameCoreLoadingState::MapSpawned),
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
