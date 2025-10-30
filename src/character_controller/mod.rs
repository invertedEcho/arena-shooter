use avian3d::{math::Quaternion, prelude::*};
use bevy::prelude::*;

use crate::{
    GRAVITY,
    game_flow::states::InGameState,
    player::{Player, camera::components::ViewModelCamera},
};

pub const CHARACTER_CAPSULE_RADIUS: f32 = 0.2;
pub const CHARACTER_CAPSULE_LENGTH: f32 = 1.3;

// Remember to update update_agent_target when this is no longer negative.
pub const LOCAL_FEET_CHARACTER: f32 =
    -((CHARACTER_CAPSULE_LENGTH + CHARACTER_CAPSULE_RADIUS * 2.0) / 2.);

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
pub struct MovementAction {
    pub direction: MovementDirection,
    pub character_controller_entity: Entity,
}

pub enum MovementDirection {
    // TODO: should be possible to just have Vec2
    Move(Vec3),
    Jump,
}

#[derive(Bundle)]
pub struct CharacterControllerBundle {
    rigid_body: RigidBody,
    collider: Collider,
    locked_axes: LockedAxes,
    movement_state: MovementState,
    colliding_entities: CollidingEntities,
    grounded: Grounded,
    ground_caster: ShapeCaster,
}

impl Default for CharacterControllerBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::Kinematic,
            collider: Collider::capsule(
                CHARACTER_CAPSULE_RADIUS,
                CHARACTER_CAPSULE_LENGTH,
            ),
            locked_axes: LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            colliding_entities: CollidingEntities::default(),
            movement_state: MovementState(MovementStateEnum::Idle),
            grounded: Grounded(true),
            ground_caster: ShapeCaster::new(
                Collider::capsule(
                    CHARACTER_CAPSULE_RADIUS,
                    CHARACTER_CAPSULE_LENGTH,
                ),
                Vec3::ZERO,
                Quaternion::default(),
                Dir3::NEG_Y,
            )
            .with_max_distance(0.1),
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
        app.add_message::<MovementAction>()
            .add_systems(
                Update,
                (
                    update_on_ground,
                    apply_gravity_over_time,
                    handle_keyboard_input_for_player,
                    handle_movement_actions_for_character_controllers,
                )
                    .run_if(in_state(InGameState::Playing)),
            )
            .add_systems(Update, apply_movement_damping)
            .add_systems(
                OnEnter(InGameState::PlayerDead),
                handle_player_dead_velocity,
            );
    }
}

fn handle_player_dead_velocity(
    mut player_velocity: Single<&mut LinearVelocity, With<Player>>,
) {
    info!("Player entered state Dead, zeroeing velocity");
    player_velocity.0 = Vec3::ZERO;
}

fn handle_keyboard_input_for_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut movement_action_writer: MessageWriter<MovementAction>,
    player_query: Single<
        (&Transform, &mut MovementState, Entity),
        With<Player>,
    >,
) {
    let (player_transform, mut movement_state, player_entity) =
        player_query.into_inner();

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

    if local_velocity == Vec3::ZERO {
        return;
    }

    let world_velocity = player_transform.rotation * local_velocity;

    movement_action_writer.write(MovementAction {
        direction: MovementDirection::Move(world_velocity),
        character_controller_entity: player_entity,
    });

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_action_writer.write(MovementAction {
            direction: MovementDirection::Jump,
            character_controller_entity: player_entity,
        });
    }
}

fn handle_movement_actions_for_character_controllers(
    mut movement_action_reader: MessageReader<MovementAction>,
    mut character_controller_query: Query<(
        &mut LinearVelocity,
        &Grounded,
        &Transform,
        Entity,
    )>,
    // TODO: i dont want this here
    player_camera_entity: Single<Entity, With<ViewModelCamera>>,
    spatial_query: SpatialQuery,
    time: Res<Time>,
) {
    for movement_action in movement_action_reader.read() {
        let direction = &movement_action.direction;
        let character_controller_entity =
            movement_action.character_controller_entity;
        let Ok((mut velocity, grounded, transform, entity)) =
            character_controller_query.get_mut(character_controller_entity)
        else {
            warn!(
                "Failed to find Character Controller by Entity {}",
                character_controller_entity
            );
            continue;
        };

        match *direction {
            MovementDirection::Jump => {
                if grounded.0 {
                    velocity.y = JUMP_VELOCITY;
                }
            }
            // TODO: should probably move the content of this block elsewhere
            MovementDirection::Move(world_velocity) => {
                let Ok(direction_from_world_velocity) =
                    Dir3::new(world_velocity)
                else {
                    velocity.x = 0.0;
                    velocity.z = 0.0;
                    return;
                };

                let ray_origin = transform.translation
                    - direction_from_world_velocity.as_vec3() * 0.025;
                let max_distance = 0.3;

                let spatial_query_filter = &SpatialQueryFilter::default()
                    .with_excluded_entities([
                        entity,
                        // TODO: should only be excluded when we have player
                        *player_camera_entity,
                    ]);

                if let Some(hit_ahead) = spatial_query.cast_shape(
                    &Collider::capsule(
                        CHARACTER_CAPSULE_RADIUS,
                        CHARACTER_CAPSULE_LENGTH,
                    ),
                    ray_origin,
                    transform.rotation,
                    direction_from_world_velocity,
                    &ShapeCastConfig {
                        max_distance,
                        ..default()
                    },
                    spatial_query_filter,
                ) {
                    // obstacle in the way, check if we can slimb it
                    // a normal is just a direction something is facing
                    let normal = hit_ahead.normal1;
                    let slope_angle = normal.angle_between(Vec3::Y);
                    let slope_climable = slope_angle < MAX_SLOPE_ANGLE;

                    if slope_climable {
                        info!("MOVEMENT: Climable slope!");
                        // this is the most important part to make the slope climbing possible.
                        // instead of trying to go straight, we slide along the ground
                        velocity.0 =
                            world_velocity.reject_from_normalized(normal);

                        // slope snapping
                        let ray_down_origin =
                            transform.translation + Vec3::Y * 0.5;
                        let ray_down_direction = Dir3::NEG_Y;
                        let max_down_distance = 1.0;

                        if let Some(hit_down) = spatial_query.cast_ray(
                            ray_down_origin,
                            ray_down_direction,
                            max_down_distance,
                            true,
                            spatial_query_filter,
                        ) {
                            let hit_down_point = ray_down_origin
                                + ray_down_direction * hit_down.distance;
                            let hit_down_y = hit_down_point.y;
                            let player_y = transform.translation.y;
                            let difference_y = hit_down_y - player_y;
                            if difference_y.abs() < 0.3 {
                                info!("Snapping character controller to slope");
                                velocity.y = difference_y / time.delta_secs();
                            }
                        }
                    } else {
                        info!(
                            "MOVEMENT: Obstacle in the way, sliding along wall"
                        );
                        // not climable, e.g. a wall. we want to slide along the wall, similar to the collide
                        // and slide algorithm
                        // the main difference is that we ignore the Y part, because its too step, so we dont
                        // want to climb up
                        let impulse =
                            world_velocity.reject_from_normalized(normal);
                        velocity.x = impulse.x;
                        velocity.z = impulse.z
                    }
                } else {
                    info!("MOVEMENT: No obstacle ahead, free movement");
                    // no obstacle ahead, free movement
                    velocity.x = world_velocity.x;
                    velocity.z = world_velocity.z;
                }
            }
        }
    }
}

/// Updates the [`Grounded`] status for character controllers.
fn update_on_ground(
    mut query: Query<(&ShapeHits, &mut Grounded, &mut LinearVelocity)>,
) {
    for (hits, mut grounded, mut velocity) in &mut query {
        let on_ground = hits.0.len() > 0;

        if grounded.0 != on_ground {
            grounded.0 = on_ground;
        }

        if on_ground && velocity.y <= 0.0 {
            velocity.y = 0.0
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

// Apply damping in the XZ Plane, basically this is deceleration over time
fn apply_movement_damping(query: Query<&mut LinearVelocity>) {
    for mut velocity in query {
        velocity.x *= 0.9;
        velocity.z *= 0.9;
    }
}
