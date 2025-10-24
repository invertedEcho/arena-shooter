use bevy::prelude::*;

use crate::shared::systems::{
    disable_culling_for_skinned_meshes, handle_despawn_timer,
};

pub mod components;
pub mod systems;

pub const BULLET_VELOCITY: f32 = 100.0;

pub const WALK_VELOCITY: f32 = 2.0;
pub const RUN_VELOCITY: f32 = 5.0;
pub const JUMP_VELOCITY: f32 = 3.0;

#[derive(Debug, Reflect, PartialEq)]
pub enum MovementState {
    Idle,
    Walking,
    Running,
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
