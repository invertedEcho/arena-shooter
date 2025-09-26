use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    GRAVITY,
    game_flow::states::{AppState, InGameState},
    player::{
        Player, PlayerMovementState,
        camera::components::PlayerCamera,
        spawn::{PLAYER_CAPSULE_LENGTH, PLAYER_CAPSULE_RADIUS},
    },
};

const PLAYER_WALK_VELOCITY: f32 = 2.0;
const PLAYER_RUN_VELOCITY: f32 = 5.0;
const PLAYER_JUMP_VELOCITY: f32 = 3.0;

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (player_movement).run_if(in_state(AppState::InGame)),
        );
    }
}

// TODO: its time to split this up, so we can also the character controller for our enemies
pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<(Entity, &mut Player, &mut LinearVelocity, &Transform)>,
    player_camera_entity: Single<Entity, With<PlayerCamera>>,
    spatial_query: SpatialQuery,
    time: Res<Time>,
    current_in_game_state: Res<State<InGameState>>,
) {
    let (entity, mut player, mut velocity, transform) = player.into_inner();

    let movement_allowed = *current_in_game_state.get() == InGameState::Playing;
    if !movement_allowed {
        **velocity = Vec3::ZERO;
        return;
    }

    let speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
        PLAYER_RUN_VELOCITY
    } else {
        PLAYER_WALK_VELOCITY
    };

    let mut local_velocity = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        local_velocity.z -= speed;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        local_velocity.x -= speed;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        local_velocity.x += speed;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        local_velocity.z += speed;
    }
    if keyboard_input.just_pressed(KeyCode::Space) && player.on_ground {
        velocity.y = PLAYER_JUMP_VELOCITY;
    }

    velocity.y -= GRAVITY * time.delta_secs();

    if let Some(_) = spatial_query.cast_shape(
        &Collider::capsule(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_LENGTH),
        transform.translation,
        transform.rotation,
        Dir3::NEG_Y,
        &ShapeCastConfig {
            max_distance: 0.5,
            ..default()
        },
        &SpatialQueryFilter::default()
            .with_excluded_entities([entity, *player_camera_entity]),
    ) {
        if velocity.y <= 0.0 {
            velocity.y = 0.0;
            player.on_ground = true;
        }
    } else {
        player.on_ground = false;
    }

    let world_velocity = transform.rotation * local_velocity;
    let maybe_normalized_world_velocity = world_velocity.try_normalize();
    let Some(normalized_world_velocity) = maybe_normalized_world_velocity
    else {
        velocity.x = 0.0;
        velocity.z = 0.0;
        player.state = PlayerMovementState::Idle;
        return;
    };

    let direction_based_on_input =
        Dir3::new_unchecked(normalized_world_velocity);

    if let Some(first_hit) = spatial_query.cast_shape(
        &Collider::capsule(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_LENGTH),
        transform.translation,
        transform.rotation,
        direction_based_on_input,
        &ShapeCastConfig {
            max_distance: 0.5,
            ..default()
        },
        &SpatialQueryFilter::default()
            .with_excluded_entities([entity, *player_camera_entity]),
    ) {
        if first_hit.distance < 0.1 {
            **velocity = Vec3::ZERO;
            player.state = PlayerMovementState::Idle;
            return;
        }
    }

    velocity.x = world_velocity.x;
    velocity.z = world_velocity.z;

    if speed == PLAYER_RUN_VELOCITY {
        player.state = PlayerMovementState::Running;
    } else if local_velocity != Vec3::ZERO {
        player.state = PlayerMovementState::Walking;
    } else {
        player.state = PlayerMovementState::Idle;
    }
}
