use bevy::prelude::*;

use crate::player::camera::{
    components::PlayerCamera,
    systems::{
        camera_follow_player, camera_orbit_player, change_mouse_motion_enabled,
        switch_between_first_and_third_person,
    },
};

pub mod components;
mod systems;

/// Everything else than player weapon is rendererd at this layer
// pub const DEFAULT_RENDER_LAYER: usize = 0;
//
// /// Used by the view model camera and the player's weapon
// pub const VIEW_MODEL_RENDER_LAYER: usize = 1;

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                camera_orbit_player,
                camera_follow_player,
                switch_between_first_and_third_person,
                change_mouse_motion_enabled,
            ),
        )
        .register_type::<PlayerCamera>();
    }
}
