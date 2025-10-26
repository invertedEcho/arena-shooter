use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    GRAVITY,
    player::{Player, camera::components::ViewModelCamera},
};

pub const CHARACTER_CAPSULE_RADIUS: f32 = 0.2;
pub const CHARACTER_CAPSULE_LENGTH: f32 = 1.3;

pub const WALK_VELOCITY: f32 = 2.0;
pub const RUN_VELOCITY: f32 = 5.0;
pub const JUMP_VELOCITY: f32 = 3.0;

pub const MAX_SLOPE_ANGLE: f32 = 45.0_f32.to_radians();

// NOTE: So far this character controller is only for the player. The code needs a bit of
// adjustment to be able to support using it for enemies. But I guess we also don't really need a
// character controller for enemies.

#[derive(Component)]
pub struct MovementState(pub MovementStateEnum);

#[derive(Debug, Reflect, PartialEq)]
pub enum MovementStateEnum {
    Idle,
    Walking,
    Running,
}

#[derive(Message)]
pub enum MovementAction {
    // TODO: should be possible to just have Vec2
    Move(Vec3),
    Jump,
}

/// Contains all needed components for a character that should be controlled by the player
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    velocity: LinearVelocity,
    rigid_body: RigidBody,
    collider: Collider,
    grounded: Grounded,
    locked_axes: LockedAxes,
    movement_state: MovementState,
    colliding_entities: CollidingEntities,
}

impl Default for CharacterControllerBundle {
    fn default() -> Self {
        Self {
            velocity: LinearVelocity::ZERO,
            rigid_body: RigidBody::Kinematic,
            collider: Collider::capsule(
                CHARACTER_CAPSULE_RADIUS,
                CHARACTER_CAPSULE_LENGTH,
            ),
            grounded: Grounded::default(),
            locked_axes: LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            movement_state: MovementState(MovementStateEnum::Idle),
            colliding_entities: CollidingEntities::default(),
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
        app.add_message::<MovementAction>().add_systems(
            Update,
            (
                update_on_ground,
                apply_gravity_over_time,
                handle_keyboard_input_for_player,
                handle_movement_actions_for_player,
            ),
        );
    }
}

fn handle_keyboard_input_for_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut movement_action_writer: MessageWriter<MovementAction>,
    player_query: Single<(&Transform, &mut MovementState), With<Player>>,
) {
    let (player_transform, mut movement_state) = player_query.into_inner();
    let speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
        RUN_VELOCITY
    } else {
        WALK_VELOCITY
    };

    let mut local_velocity = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        local_velocity.z -= 1.0 * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        local_velocity.x -= 1.0 * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        local_velocity.x += 1.0 * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        local_velocity.z += 1.0 * speed;
    }

    let world_velocity = player_transform.rotation * local_velocity;

    movement_action_writer.write(MovementAction::Move(world_velocity));
    if local_velocity.x == 0.0 && local_velocity.z == 0.0 {
        if movement_state.0 != MovementStateEnum::Idle {
            movement_state.0 = MovementStateEnum::Idle;
        }
    } else if speed == RUN_VELOCITY {
        if movement_state.0 != MovementStateEnum::Running {
            movement_state.0 = MovementStateEnum::Running;
        }
    } else if speed == WALK_VELOCITY {
        if movement_state.0 != MovementStateEnum::Walking {
            movement_state.0 = MovementStateEnum::Walking;
        }
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_action_writer.write(MovementAction::Jump);
    }
}

fn handle_movement_actions_for_player(
    mut movement_action_reader: MessageReader<MovementAction>,
    player_query: Single<
        (&mut LinearVelocity, &Grounded, &Transform, Entity),
        With<Player>,
    >,
    player_camera_entity: Single<Entity, With<ViewModelCamera>>,
    spatial_query: SpatialQuery,
    time: Res<Time>,
) {
    let (mut player_velocity, player_grounded, player_transform, player_entity) =
        player_query.into_inner();
    for movement_action in movement_action_reader.read() {
        match movement_action {
            MovementAction::Jump => {
                if player_grounded.0 {
                    player_velocity.y = JUMP_VELOCITY;
                }
            }
            // TODO: should probably move the content of this block elsewhere
            MovementAction::Move(world_velocity) => {
                let Ok(direction_from_world_velocity) =
                    Dir3::new(*world_velocity)
                else {
                    player_velocity.x = 0.0;
                    player_velocity.z = 0.0;
                    return;
                };

                let ray_origin = player_transform.translation
                    - direction_from_world_velocity.as_vec3() * 0.025;
                let max_distance = 0.3;

                if let Some(hit_ahead) = spatial_query.cast_shape(
                    &Collider::capsule(
                        CHARACTER_CAPSULE_RADIUS,
                        CHARACTER_CAPSULE_LENGTH,
                    ),
                    ray_origin,
                    player_transform.rotation,
                    direction_from_world_velocity,
                    &ShapeCastConfig {
                        max_distance,
                        ..default()
                    },
                    &SpatialQueryFilter::default().with_excluded_entities([
                        player_entity,
                        *player_camera_entity,
                    ]),
                ) {
                    // obstacle in the way, check if we can slimb it
                    // a normal is just a direction something is facing
                    let normal = hit_ahead.normal1;
                    let slope_angle = normal.angle_between(Vec3::Y);
                    let slope_climable = slope_angle < MAX_SLOPE_ANGLE;

                    if slope_climable {
                        debug!("MOVEMENT: Climable slope!");
                        // this is the most important part to make the slope climbing possible.
                        // instead of trying to go straight, we slide along the ground
                        player_velocity.0 =
                            world_velocity.reject_from_normalized(normal);

                        // slope snapping
                        let ray_down_origin =
                            player_transform.translation + Vec3::Y * 0.5;
                        let ray_down_direction = Dir3::NEG_Y;
                        let max_down_distance = 1.0;

                        if let Some(hit_down) = spatial_query.cast_ray(
                            ray_down_origin,
                            ray_down_direction,
                            max_down_distance,
                            true,
                            &SpatialQueryFilter::default()
                                .with_excluded_entities([
                                    player_entity,
                                    *player_camera_entity,
                                ]),
                        ) {
                            let hit_down_point = ray_down_origin
                                + ray_down_direction * hit_down.distance;
                            let hit_down_y = hit_down_point.y;
                            let player_y = player_transform.translation.y;
                            let difference_y = hit_down_y - player_y;
                            if difference_y.abs() < 0.3 {
                                debug!("Snapping player to slope");
                                player_velocity.y =
                                    difference_y / time.delta_secs();
                            }
                        }
                    } else {
                        debug!(
                            "MOVEMENT: Obstacle in the way, sliding along wall"
                        );
                        // not climable, e.g. a wall. we want to slide along the wall, similar to the collide
                        // and slide algorithm
                        // the main difference is that we ignore the Y part, because its too step, so we dont
                        // want to climb up
                        let impulse =
                            world_velocity.reject_from_normalized(normal);
                        player_velocity.x = impulse.x;
                        player_velocity.z = impulse.z
                    }
                } else {
                    debug!("MOVEMENT: No obstacle ahead, free movement");
                    // no obstacle ahead, free movement
                    player_velocity.x = world_velocity.x;
                    player_velocity.z = world_velocity.z;
                }
            }
        }
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
