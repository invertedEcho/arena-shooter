use avian3d::prelude::*;
use bevy::prelude::*;

use crate::GRAVITY;

const CAPSULE_RADIUS: f32 = 0.2;
const CAPSULE_LENGTH: f32 = 1.3;

/// Contains all needed components for a character that should be controlled by the player
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    velocity: LinearVelocity,
    rigid_body: RigidBody,
    collider: Collider,
    grounded: Grounded,
    locked_axes: LockedAxes,
}

impl Default for CharacterControllerBundle {
    fn default() -> Self {
        Self {
            velocity: LinearVelocity::ZERO,
            rigid_body: RigidBody::Kinematic,
            collider: Collider::capsule(CAPSULE_RADIUS, CAPSULE_LENGTH),
            grounded: Grounded::default(),
            locked_axes: LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
        }
    }
}

#[derive(Component)]
pub struct Grounded(pub bool);

impl Default for Grounded {
    fn default() -> Self {
        Self(true)
    }
}

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
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

fn check_shape_hit_ahead(
    spatial_query: SpatialQuery,
    shape: &Collider,
    origin: Vec3,
    shape_rotation: Quat,
    direction: Dir3,
    excluded_entities: Vec<Entity>,
) -> bool {
    spatial_query
        .cast_shape(
            shape,
            origin,
            shape_rotation,
            direction,
            &ShapeCastConfig {
                max_distance: 0.2,
                ..default()
            },
            &SpatialQueryFilter::default()
                .with_excluded_entities(excluded_entities),
        )
        .is_some()
}
