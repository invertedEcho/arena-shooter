use bevy::prelude::*;

use crate::common::systems::{
    disable_culling_for_skinned_meshes, handle_despawn_timer,
};

pub mod components;
pub mod systems;

pub const BULLET_VELOCITY: f32 = 100.0;

// TODO: this should just be player arm weapon animation state
#[derive(Debug, Reflect, PartialEq)]
pub enum MovementState {
    Idle,
    Walking,
    Running,
    // TODO: i dont know if i like this but this is so when we play shoot animation during walking,
    // we know that we must play different animation again and switch
    Else,
}

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_despawn_timer, disable_culling_for_skinned_meshes),
        );
    }
}
