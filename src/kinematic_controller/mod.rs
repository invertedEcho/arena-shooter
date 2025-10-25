use avian3d::prelude::*;
use bevy::prelude::*;

use crate::GRAVITY;

// so total length is 1.9m = 1.4 + 0.25 * 2
const CAPSULE_RADIUS: f32 = 0.25;
const CAPSULE_LENGTH: f32 = 1.4;

#[derive(Component)]
pub struct KinematicController {
    pub on_ground: bool,
}

impl Default for KinematicController {
    fn default() -> Self {
        Self { on_ground: true }
    }
}

pub struct KinematicControllerPlugin;

impl Plugin for KinematicControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                add_required_components_for_kinematic_controllers,
                update_on_ground,
                apply_gravity_over_time,
            ),
        );
    }
}

fn add_required_components_for_kinematic_controllers(
    mut commands: Commands,
    query: Query<Entity, Added<KinematicController>>,
) {
    for entity in query {
        commands.entity(entity).insert((
            RigidBody::Kinematic,
            LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            Collider::capsule(CAPSULE_RADIUS, CAPSULE_LENGTH),
            LinearVelocity::ZERO,
            CollidingEntities::default(),
        ));
    }
}

fn update_on_ground(
    query: Query<(
        &Transform,
        Entity,
        &mut LinearVelocity,
        &mut KinematicController,
    )>,
    spatial_query: SpatialQuery,
) {
    for (transform, entity, mut velocity, mut kinematic_controller) in query {
        let on_ground = spatial_query
            .cast_shape(
                &Collider::capsule(CAPSULE_RADIUS, CAPSULE_LENGTH),
                transform.translation,
                transform.rotation,
                Dir3::NEG_Y,
                &ShapeCastConfig {
                    max_distance: 0.1,
                    ..default()
                },
                &SpatialQueryFilter::default().with_excluded_entities([entity]),
            )
            .is_some();
        if kinematic_controller.on_ground != on_ground {
            kinematic_controller.on_ground = on_ground;
        }

        if on_ground && velocity.y <= 0.0 {
            velocity.y = 0.0;
        }
    }
}

fn apply_gravity_over_time(
    query: Query<(&KinematicController, &mut LinearVelocity)>,
    time: Res<Time>,
) {
    for (kinematic_controller, mut velocity) in query {
        if !kinematic_controller.on_ground {
            velocity.y -= GRAVITY * time.delta_secs();
        }
    }
}
