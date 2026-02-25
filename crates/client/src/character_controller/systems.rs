use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::Controlled;
use shared::{
    GRAVITY, Medkit,
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS, JUMP_VELOCITY,
        RUN_VELOCITY, WALK_VELOCITY, apply_collide_and_slide,
        components::{CharacterController, Grounded},
    },
    enemy::components::Enemy,
    player::PlayerState,
};

use crate::{
    character_controller::messages::{MovementAction, MovementDirection},
    player::camera::components::{PlayerCameraState, WorldCamera},
    utils::query_filters::OurPlayerFilter,
};

pub fn handle_keyboard_input_for_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut movement_action_writer: MessageWriter<MovementAction>,
    player_query: Single<(Entity, &PlayerCameraState, &PlayerState)>,
    camera_transform: Single<&Transform, With<WorldCamera>>,
) {
    let (player_entity, player_camera_state, player_state) =
        player_query.into_inner();

    if *player_camera_state == PlayerCameraState::FreeCam {
        return;
    }

    let sprint = keyboard_input.pressed(KeyCode::ShiftLeft);
    let reloading = player_state.reloading;

    let speed = if sprint && !reloading {
        RUN_VELOCITY
    } else {
        WALK_VELOCITY
    };

    let forward_camera = camera_transform.forward();
    let right = camera_transform.right();

    let Ok(forward_camera) =
        Dir3::from_xyz(forward_camera.x, 0.0, forward_camera.z)
    else {
        return;
    };
    let Ok(right) = Dir3::from_xyz(right.x, 0.0, right.z) else {
        return;
    };

    let mut desired_velocity = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        desired_velocity += forward_camera * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        desired_velocity -= right * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        desired_velocity += right * speed;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        desired_velocity -= forward_camera * speed;
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_action_writer.write(MovementAction {
            desired_velocity: MovementDirection::Jump,
            character_controller_entity: player_entity,
            sprinting: sprint,
        });
    }

    if desired_velocity != Vec3::ZERO {
        movement_action_writer.write(MovementAction {
            desired_velocity: MovementDirection::Move(desired_velocity),
            character_controller_entity: player_entity,
            sprinting: sprint,
        });
    }
}

pub fn apply_movement_damping(
    player_query: Single<(&mut LinearVelocity, &Grounded), OurPlayerFilter>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let (mut player_velocity, grounded) = player_query.into_inner();

    let any_movement_input = keyboard_input.pressed(KeyCode::KeyW)
        || keyboard_input.pressed(KeyCode::KeyA)
        || keyboard_input.pressed(KeyCode::KeyS)
        || keyboard_input.pressed(KeyCode::KeyD);

    if !any_movement_input && grounded.0 {
        player_velocity.x *= 0.8;
        player_velocity.z *= 0.8;
    }
}

pub fn handle_movement_actions_for_character_controllers(
    mut movement_action_reader: MessageReader<MovementAction>,
    mut character_controller_query: Query<
        (&mut LinearVelocity, &Grounded, &Transform),
        With<CharacterController>,
    >,
    mut spatial_query: SpatialQuery,
    time: Res<Time>,
    medkit_query: Query<Entity, With<Medkit>>,
) {
    for movement_action in movement_action_reader.read() {
        let sprinting = movement_action.sprinting;
        let direction = &movement_action.desired_velocity;
        let character_controller_entity =
            movement_action.character_controller_entity;
        let Ok((mut velocity, grounded, transform)) =
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
                // This is pretty naive, but theoretically it works because we have the
                // `check_above_head` system. In the future, we most likely just want a
                // depenetration system which finds the shortest way out of the collider that we
                // are stuck in.
                if grounded.0 {
                    velocity.y = JUMP_VELOCITY;
                }
            }
            MovementDirection::Move(desired_velocity) => {
                // exclude medkits because we want to be able to walk through medkits
                let excluded_entities: Vec<Entity> = medkit_query
                    .iter()
                    .chain(std::iter::once(character_controller_entity))
                    .collect();

                let spatial_query_filter = &SpatialQueryFilter::default()
                    .with_excluded_entities(excluded_entities.clone());

                apply_collide_and_slide(
                    &mut velocity,
                    desired_velocity,
                    transform,
                    &mut spatial_query,
                    spatial_query_filter,
                    time.delta_secs(),
                    0,
                    sprinting,
                );
            }
        }
    }
}

/// Updates the [`Grounded`] component
pub fn update_grounded(
    mut query: Query<(&ShapeHits, &mut Grounded), EntitiesRelevantForGravity>,
) {
    for (hits, mut grounded) in &mut query {
        let on_ground = !hits.0.is_empty();

        if grounded.0 != on_ground {
            grounded.0 = on_ground;
        }
    }
}

type EntitiesRelevantForGravity = Or<(With<Enemy>, With<CharacterController>)>;

pub fn apply_gravity(
    query: Query<(&Grounded, &mut LinearVelocity), EntitiesRelevantForGravity>,
    time: Res<Time>,
) {
    const MAX_VERTICAL_VELOCITY: f32 = 15.0;
    for (grounded, mut velocity) in query {
        if !grounded.0 {
            velocity.y -= GRAVITY * time.delta_secs();
            velocity.y = velocity
                .y
                .clamp(-MAX_VERTICAL_VELOCITY, MAX_VERTICAL_VELOCITY);
        } else {
            // if we are grounded and have negative y velocity, stop downwards vertical movement
            // so we dont sink into the ground
            if velocity.y < 0.0 {
                velocity.y = 0.0
            }
        }
    }
}

pub fn zero_player_velocity(
    mut player_velocity: Single<&mut LinearVelocity, With<Controlled>>,
) {
    player_velocity.x = 0.0;
    player_velocity.z = 0.0;
}

pub fn check_above_head(
    query: Query<
        (Entity, &mut LinearVelocity, &Transform, &Grounded),
        With<CharacterController>,
    >,
    spatial_query: SpatialQuery,
) {
    for (entity_itself, mut velocity, transform, grounded) in query {
        if grounded.0 {
            continue;
        };
        let Some(_) = spatial_query.cast_shape(
            &Collider::capsule(
                CHARACTER_CAPSULE_RADIUS,
                CHARACTER_CAPSULE_LENGTH,
            ),
            transform.translation,
            Quat::IDENTITY,
            Dir3::Y,
            // TODO: investigate whether we can further decrease this value
            &ShapeCastConfig::default().with_max_distance(0.1),
            &SpatialQueryFilter::default()
                .with_excluded_entities([entity_itself]),
        ) else {
            continue;
        };

        // if there is something above the current shape, stop vertical movement, to prevent
        // clipping into ceilings
        velocity.y = -0.5;
    }
}
