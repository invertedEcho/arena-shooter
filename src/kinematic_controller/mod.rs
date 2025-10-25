use avian3d::prelude::*;
use bevy::prelude::*;

use crate::GRAVITY;

// so total length is 1.9m = 1.4 + 0.25 * 2
const CHARACTER_CAPSULE_RADIUS: f32 = 0.25;
const CHARACTER_CAPSULE_LENGTH: f32 = 1.4;

#[derive(Bundle)]
pub struct CharacterControllerBundle {
    rigid_body: RigidBody,
    velocity: LinearVelocity,
    colliding_entities: CollidingEntities,
    collider: Collider,
    locked_axes: LockedAxes,
    grounded: Grounded,
}

impl Default for CharacterControllerBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::Kinematic,
            velocity: LinearVelocity::ZERO,
            colliding_entities: CollidingEntities::default(),
            collider: Collider::capsule(
                CHARACTER_CAPSULE_RADIUS,
                CHARACTER_CAPSULE_LENGTH,
            ),
            locked_axes: LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            grounded: Grounded(true),
        }
    }
}

#[derive(Component)]
pub struct Grounded(pub bool);

pub struct KinematicControllerPlugin;

impl Plugin for KinematicControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_on_ground, apply_gravity_over_time));
    }
}

fn update_on_ground(
    query: Query<(&Transform, Entity, &mut LinearVelocity, &mut Grounded)>,
    spatial_query: SpatialQuery,
) {
    for (transform, entity, mut velocity, mut grounded) in query {
        let on_ground = spatial_query
            .cast_shape(
                &Collider::capsule(
                    CHARACTER_CAPSULE_RADIUS,
                    CHARACTER_CAPSULE_LENGTH,
                ),
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
        if grounded.0 != on_ground {
            grounded.0 = on_ground;
        }

        if on_ground && velocity.y <= 0.0 {
            velocity.y = 0.0;
        }
    }
}

fn apply_gravity_over_time(
    query: Query<(&Grounded, &mut LinearVelocity)>,
    time: Res<Time>,
) {
    for (grounded, mut velocity) in query {
        if !grounded.0 {
            velocity.y -= GRAVITY * time.delta_secs();
        }
    }
}
