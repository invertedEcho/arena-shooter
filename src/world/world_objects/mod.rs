use bevy::prelude::*;

use medkit::MedkitSpawnLocation;

use crate::{
    game_flow::states::{InGameState, LoadingGameSubState},
    world::world_objects::medkit::{
        detect_collision_medkit_with_player, rotate_and_float_medkits,
        spawn_medkits,
    },
};

mod medkit;

pub struct WorldObjectsPlugin;

impl Plugin for WorldObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MedkitSpawnLocation>()
            .add_systems(
                OnEnter(LoadingGameSubState::MapLoadedWithDependencies),
                spawn_medkits,
            )
            .add_systems(
                Update,
                (
                    rotate_and_float_medkits,
                    detect_collision_medkit_with_player,
                )
                    .run_if(in_state(InGameState::Playing)),
            );
    }
}
