use bevy::prelude::*;

use crate::ground_detection::systems::{
    detect_ground_collision, setup_ground_detection, update_on_ground,
};

pub mod components;
mod systems;

pub struct GroundDetectionPlugin;

impl Plugin for GroundDetectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                setup_ground_detection,
                detect_ground_collision,
                update_on_ground,
            ),
        );
    }
}
