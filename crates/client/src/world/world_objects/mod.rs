use bevy::prelude::*;
use game_core::ServerLoadingState;

use crate::{
    game_flow::states::InGameState,
    world::world_objects::medkit::{
        activate_medkits_over_time, detect_collision_medkit_with_player,
        rotate_and_float_medkits, spawn_medkits, tick_respawn_timer_medkits,
    },
};

pub mod medkit;

pub struct WorldObjectsPlugin;

impl Plugin for WorldObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ServerLoadingState::MapSpawned), spawn_medkits)
            .add_systems(
                Update,
                (
                    rotate_and_float_medkits,
                    detect_collision_medkit_with_player,
                    activate_medkits_over_time,
                    tick_respawn_timer_medkits,
                )
                    .run_if(in_state(InGameState::Playing)),
            );
    }
}
