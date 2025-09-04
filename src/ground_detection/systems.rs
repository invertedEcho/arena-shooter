use std::collections::HashSet;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    ground_detection::components::{GroundDetection, GroundSensor},
    world::components::Ground,
};

pub fn setup_ground_detection(
    mut commands: Commands,
    ground_detection_query: Query<Entity, Added<GroundDetection>>,
) {
    for ground_detection_entity in ground_detection_query {
        commands
            .entity(ground_detection_entity)
            .with_children(|builder| {
                builder.spawn_empty().insert((
                    CollisionEventsEnabled,
                    Collider::cuboid(0.2, 0.2, 0.2),
                    Transform::from_xyz(0.0, -0.8, 0.0),
                    GroundSensor {
                        ground_detection_entity,
                        intersecting_ground_entities: HashSet::new(),
                    },
                    Sensor,
                ));
            });
    }
}

pub fn detect_ground_collision(
    mut collision_started_event_reader: EventReader<CollisionStarted>,
    mut collision_ended_event_reader: EventReader<CollisionEnded>,
    mut ground_sensor_query: Query<&mut GroundSensor>,
    ground_query: Query<Entity, With<Ground>>,
) {
    for collision_started_event in collision_started_event_reader.read() {
        let first_entity = collision_started_event.0;
        let second_entity = collision_started_event.1;

        if ground_query.contains(first_entity) {
            if let Ok(mut ground_sensor) = ground_sensor_query.get_mut(second_entity) {
                ground_sensor
                    .intersecting_ground_entities
                    .insert(first_entity);
            }
        } else if ground_query.contains(second_entity) {
            if let Ok(mut ground_sensor) = ground_sensor_query.get_mut(first_entity) {
                ground_sensor
                    .intersecting_ground_entities
                    .insert(second_entity);
            }
        }
    }

    for collision_ended_event in collision_ended_event_reader.read() {
        let first_entity = collision_ended_event.0;
        let second_entity = collision_ended_event.1;

        if ground_query.contains(first_entity) {
            if let Ok(mut ground_sensor) = ground_sensor_query.get_mut(second_entity) {
                ground_sensor
                    .intersecting_ground_entities
                    .remove(&first_entity);
            }
        } else if ground_query.contains(second_entity) {
            if let Ok(mut ground_sensor) = ground_sensor_query.get_mut(first_entity) {
                ground_sensor
                    .intersecting_ground_entities
                    .remove(&second_entity);
            }
        }
    }
}

pub fn update_on_ground(
    mut ground_detectors: Query<&mut GroundDetection>,
    ground_sensors: Query<&GroundSensor, Changed<GroundSensor>>,
) {
    for sensor in &ground_sensors {
        if let Ok(mut ground_detection) = ground_detectors.get_mut(sensor.ground_detection_entity) {
            ground_detection.on_ground = !sensor.intersecting_ground_entities.is_empty();
        }
    }
}
