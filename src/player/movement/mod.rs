use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    game_flow::AppState,
    player::{
        Player, PlayerMovementState,
        camera::components::PlayerCamera,
        spawn::{PLAYER_CAPSULE_LENGTH, PLAYER_CAPSULE_RADIUS},
    },
};

const PLAYER_WALK_SPEED: f32 = 2.0;
const PLAYER_RUN_SPEED: f32 = 5.0;

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (player_movement).run_if(in_state(AppState::InGame)),
        );
    }
}

pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<(Entity, &mut Player, &mut LinearVelocity, &Transform)>,
    player_camera_entity: Single<Entity, With<PlayerCamera>>,
    spatial_query: SpatialQuery,
) {
    let (player_entity, mut player, mut velocity, player_transform) =
        player.into_inner();

    // okay so its not that easy.
    info!("player forward: {:?}", player_transform.forward());
    if let Some(first_hit) = spatial_query.cast_shape(
        &Collider::capsule(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_LENGTH),
        player_transform.translation,
        player_transform.rotation,
        player_transform.forward(),
        &ShapeCastConfig {
            max_distance: 0.5,
            ..default()
        },
        &SpatialQueryFilter::default()
            .with_excluded_entities([player_entity, *player_camera_entity]),
    ) {
        if first_hit.distance < 0.1 {
            info!(
                "disallowing movement as there is a obstacle in direction of player: {:?}",
                first_hit
            );
            **velocity = Vec3::ZERO;
            info!("disallowing movement");
            player.state = PlayerMovementState::Idle;
            return;
        }
    }

    let speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
        PLAYER_RUN_SPEED
    } else {
        PLAYER_WALK_SPEED
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
    // TODO: cast ray below us
    if keyboard_input.just_pressed(KeyCode::Space) {
        local_velocity.y += 3.0;
    }

    let world_velocity = player_transform.rotation * local_velocity;
    velocity.x = world_velocity.x;
    velocity.z = world_velocity.z;

    if speed == PLAYER_RUN_SPEED {
        player.state = PlayerMovementState::Running;
    } else if local_velocity != Vec3::ZERO {
        player.state = PlayerMovementState::Walking;
    } else {
        player.state = PlayerMovementState::Idle;
    }
}
