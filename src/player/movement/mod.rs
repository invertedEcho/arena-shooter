use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS, Grounded,
        MAX_SLOPE_ANGLE,
    },
    game_flow::states::{AppState, InGameState},
    player::{
        Player,
        animate::{ArmWithWeaponAnimation, PlayArmWithWeaponAnimationMessage},
        camera::components::ViewModelCamera,
        shooting::components::PlayerWeapon,
    },
    shared::{JUMP_VELOCITY, MovementState, RUN_VELOCITY, WALK_VELOCITY},
};

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_movement,
                setup_player_movement_state_for_added_players,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct PlayerMovementState(pub MovementState);

pub fn setup_player_movement_state_for_added_players(
    mut commands: Commands,
    added_players: Query<Entity, Added<Player>>,
) {
    for added_player in added_players {
        commands
            .entity(added_player)
            .insert(PlayerMovementState(MovementState::Idle));
    }
}

// TODO: its time to split this up, so we can also the character controller for our enemies
pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<(
        Entity,
        &mut LinearVelocity,
        &Transform,
        &mut PlayerMovementState,
        &PlayerWeapon,
        &Grounded,
    )>,
    player_camera_entity: Single<Entity, With<ViewModelCamera>>,
    spatial_query: SpatialQuery,
    current_in_game_state: Res<State<InGameState>>,
    mut play_player_arm_weapon_animation_message_writer: MessageWriter<
        PlayArmWithWeaponAnimationMessage,
    >,
    time: Res<Time>,
) {
    let (
        entity,
        mut velocity,
        player_transform,
        mut player_movement_state,
        player_weapon,
        grounded,
    ) = player.into_inner();

    let currently_playing =
        *current_in_game_state.get() == InGameState::Playing;
    if !currently_playing {
        **velocity = Vec3::ZERO;
        if player_movement_state.0 != MovementState::Idle {
            player_movement_state.0 = MovementState::Idle;
            play_player_arm_weapon_animation_message_writer.write(
                PlayArmWithWeaponAnimationMessage {
                    animation_type: ArmWithWeaponAnimation::Idle,
                    repeat: true,
                    block_until_done: false,
                },
            );
        }
        return;
    }

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
    if keyboard_input.just_pressed(KeyCode::Space) && grounded.0 {
        velocity.y = JUMP_VELOCITY;
    }

    if local_velocity == Vec3::ZERO {
        velocity.x = 0.0;
        velocity.z = 0.0;
        if player_movement_state.0 != MovementState::Idle {
            player_movement_state.0 = MovementState::Idle;
            play_player_arm_weapon_animation_message_writer.write(
                PlayArmWithWeaponAnimationMessage {
                    animation_type: ArmWithWeaponAnimation::Idle,
                    repeat: true,
                    block_until_done: false,
                },
            );
        };
        return;
    }

    let world_velocity = player_transform.rotation * local_velocity;
    let Ok(direction_from_world_velocity) = Dir3::new(world_velocity) else {
        return;
    };

    let ray_origin = player_transform.translation
        - direction_from_world_velocity.as_vec3() * 0.025;
    let max_distance = 0.3;

    if let Some(hit_ahead) = spatial_query.cast_shape(
        &Collider::capsule(CHARACTER_CAPSULE_RADIUS, CHARACTER_CAPSULE_LENGTH),
        ray_origin,
        player_transform.rotation,
        direction_from_world_velocity,
        &ShapeCastConfig {
            max_distance,
            ..default()
        },
        &SpatialQueryFilter::default()
            .with_excluded_entities([entity, *player_camera_entity]),
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
            velocity.0 = world_velocity.reject_from_normalized(normal);

            // slope snapping
            let ray_down_origin = player_transform.translation + Vec3::Y * 0.5;
            let ray_down_direction = Dir3::NEG_Y;
            let max_down_distance = 1.0;

            if let Some(hit_down) = spatial_query.cast_ray(
                ray_down_origin,
                ray_down_direction,
                max_down_distance,
                true,
                &SpatialQueryFilter::default()
                    .with_excluded_entities([entity, *player_camera_entity]),
            ) {
                let hit_down_point =
                    ray_down_origin + ray_down_direction * hit_down.distance;
                let ground_y = hit_down_point.y;
                let current_y = player_transform.translation.y;
                let diff_y = ground_y - current_y;
                if diff_y.abs() < 0.3 {
                    let res = diff_y / time.delta_secs();
                    info!(
                        "MOVEMENT: y velocity needs to be adjusted, setting \
                         to: {}",
                        res
                    );
                    velocity.y = res;
                }
            }
        } else {
            info!("MOVEMENT: Obstacle in the way, sliding along wall");
            // not climable, e.g. a wall. we want to slide along the wall, similar to the collide
            // and slide algorithm
            // the main difference is that we ignore the Y part, because its too step, so we dont
            // want to climb up
            let impulse = world_velocity.reject_from_normalized(normal);
            velocity.x = impulse.x;
            velocity.z = impulse.z
        }
    } else {
        info!("MOVEMENT: No obstacle ahead, free movement");
        // no obstacle ahead, free movement
        velocity.x = world_velocity.x;
        velocity.z = world_velocity.z;
    }

    if player_weapon.reloading {
        return;
    }

    if speed == RUN_VELOCITY {
        if player_movement_state.0 != MovementState::Running {
            player_movement_state.0 = MovementState::Running;
            play_player_arm_weapon_animation_message_writer.write(
                PlayArmWithWeaponAnimationMessage {
                    animation_type: ArmWithWeaponAnimation::Run,
                    repeat: true,
                    block_until_done: false,
                },
            );
        }
    } else if local_velocity.x != 0.0 || local_velocity.z != 0.0 {
        if player_movement_state.0 != MovementState::Walking {
            player_movement_state.0 = MovementState::Walking;
            play_player_arm_weapon_animation_message_writer.write(
                PlayArmWithWeaponAnimationMessage {
                    animation_type: ArmWithWeaponAnimation::Walk,
                    repeat: true,
                    block_until_done: false,
                },
            );
        }
    } else if local_velocity.x == 0.0 && local_velocity.z == 0.0 {
        if player_movement_state.0 != MovementState::Idle {
            player_movement_state.0 = MovementState::Idle;
            play_player_arm_weapon_animation_message_writer.write(
                PlayArmWithWeaponAnimationMessage {
                    animation_type: ArmWithWeaponAnimation::Idle,
                    repeat: true,
                    block_until_done: false,
                },
            );
        }
    }
}
