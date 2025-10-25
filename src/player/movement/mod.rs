use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    game_flow::states::{AppState, InGameState},
    kinematic_controller::KinematicController,
    player::{
        Player,
        animate::{ArmWithWeaponAnimation, PlayArmWithWeaponAnimationMessage},
        camera::components::ViewModelCamera,
        shooting::components::PlayerWeapon,
        spawn::{PLAYER_CAPSULE_LENGTH, PLAYER_CAPSULE_RADIUS},
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
        &KinematicController,
    )>,
    player_camera_entity: Single<Entity, With<ViewModelCamera>>,
    spatial_query: SpatialQuery,
    current_in_game_state: Res<State<InGameState>>,
    mut play_player_arm_weapon_animation_message_writer: MessageWriter<
        PlayArmWithWeaponAnimationMessage,
    >,
) {
    let (
        entity,
        mut velocity,
        player_transform,
        mut player_movement_state,
        player_weapon,
        kinematic_controller,
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
    if keyboard_input.just_pressed(KeyCode::Space)
        && kinematic_controller.on_ground
    {
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
    let Ok(direction) = Dir3::new(world_velocity) else {
        return;
    };

    if let Some(_) = spatial_query.cast_shape(
        &Collider::capsule(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_LENGTH),
        player_transform.translation - direction.as_vec3() * 0.025,
        player_transform.rotation,
        direction,
        &ShapeCastConfig {
            max_distance: 0.2,
            ..default()
        },
        &SpatialQueryFilter::default()
            .with_excluded_entities([entity, *player_camera_entity]),
    ) {
        velocity.x = 0.0;
        velocity.z = 0.0;
        return;
    }

    velocity.x = world_velocity.x;
    velocity.z = world_velocity.z;

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
