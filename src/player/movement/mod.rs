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
    player: Single<(
        Entity,
        &mut Player,
        &mut LinearVelocity,
        &Transform,
        &ShapeCaster,
        &ShapeHits,
    )>,
    player_camera_entity: Single<Entity, With<PlayerCamera>>,
    spatial_query: SpatialQuery,
    time: Res<Time>,
    current_in_game_state: Res<State<InGameState>>,
) {
    let (entity, mut player, mut velocity, transform, shape_caster, shape_hits) =
        player.into_inner();

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
        local_velocity.z -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        local_velocity.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        local_velocity.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        local_velocity.z += 1.0;
    }
    if keyboard_input.just_pressed(KeyCode::Space) && player.on_ground {
        velocity.y = PLAYER_JUMP_VELOCITY;
    }

    if local_velocity == Vec3::ZERO {
        velocity.x = 0.0;
        velocity.z = 0.0;
        player.state = PlayerMovementState::Idle;
        return;
    }

    velocity.y -= GRAVITY * time.delta_secs();

    // check if we are on ground
    if let Some(_) = spatial_query.cast_shape(
        &Collider::capsule(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_LENGTH),
        transform.translation,
        transform.rotation,
        Dir3::NEG_Y,
        &ShapeCastConfig {
            max_distance: 0.1,
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

    for ray_cast_hit in shape_hits.iter() {
        return;
    }

    // let mut ray_cast_hits = shape_hits.iter();
    // ray_cast_hits = ray_cast_hits.collect();
    //
    // // check if there is an obstacle in the direction the player is trying to go
    // if let Some(first_hit) = shape_caster(
    //     &Collider::capsule(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_LENGTH),
    //     transform.translation,
    //     transform.rotation,
    //     Dir3::new(local_velocity).unwrap(),
    //     &ShapeCastConfig {
    //         max_distance: 0.1,
    //         ..default()
    //     },
    //     &SpatialQueryFilter::default()
    //         .with_excluded_entities([entity, *player_camera_entity]),
    // ) {
    //     info!("first_hit: {:?}", first_hit);
    //     **velocity = Vec3::ZERO;
    //     player.state = PlayerMovementState::Idle;
    //     return;
    // }
    //
    let world_velocity = transform.rotation * local_velocity * speed;
    let Some(normalized_world_velocity) = world_velocity.try_normalize() else {
        return;
    };

    velocity.x = normalized_world_velocity.x;
    velocity.z = normalized_world_velocity.z;

    if speed == PLAYER_RUN_VELOCITY {
        player.state = PlayerMovementState::Running;
    } else if local_velocity != Vec3::ZERO {
        player.state = PlayerMovementState::Walking;
    } else {
        player.state = PlayerMovementState::Idle;
    }
}
