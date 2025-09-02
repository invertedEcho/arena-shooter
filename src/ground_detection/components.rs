use std::collections::HashSet;

use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct GroundDetection {
    pub on_ground: bool,
}

impl Default for GroundDetection {
    fn default() -> Self {
        GroundDetection { on_ground: true }
    }
}

#[derive(Component)]
pub struct GroundSensor {
    pub ground_detection_entity: Entity,
    pub intersecting_ground_entities: HashSet<Entity>,
}
