use bevy::prelude::*;

use crate::shared::systems::{
    disable_culling_for_skinned_meshes, handle_despawn_timer,
};

pub mod components;
pub mod systems;

pub const BULLET_VELOCITY: f32 = 100.0;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_despawn_timer, disable_culling_for_skinned_meshes),
        );
    }
}
