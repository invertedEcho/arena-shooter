use avian3d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};

use crate::{
    GRAVITY,
    common::MovementState,
    game_flow::states::{AppState, InGameState},
    player::{
        Player,
        animate::{ArmWithWeaponAnimation, PlayArmWithWeaponAnimationMessage},
        camera::components::ViewModelCamera,
        shooting::components::PlayerWeapon,
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
            (
                player_movement,
                update_player_on_ground,
                apply_gravity_over_time,
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
        &Player,
        &mut LinearVelocity,
        &Transform,
        &mut PlayerMovementState,
        &PlayerWeapon,
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
        player,
        mut velocity,
        player_transform,
        mut player_movement_state,
        player_weapon,
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
        PLAYER_RUN_VELOCITY
    } else {
        PLAYER_WALK_VELOCITY
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
    if keyboard_input.just_pressed(KeyCode::Space) && player.on_ground {
        velocity.y = PLAYER_JUMP_VELOCITY;
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

    if speed == PLAYER_RUN_VELOCITY {
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

fn apply_gravity_over_time(
    mut player_query: Single<(&Player, &mut LinearVelocity)>,
    time: Res<Time>,
) {
    let player = player_query.0;
    let player_velocity = &mut player_query.1;

    if !player.on_ground {
        player_velocity.y -= GRAVITY * time.delta_secs();
    }
}

fn update_player_on_ground(
    players: Query<(&mut Player, &Transform, Entity, &mut LinearVelocity)>,
    spatial_query: SpatialQuery,
) {
    for (mut player, transform, player_entity, mut player_velocity) in players {
        let on_ground = spatial_query
            .cast_shape(
                &Collider::capsule(
                    PLAYER_CAPSULE_RADIUS,
                    PLAYER_CAPSULE_LENGTH,
                ),
                transform.translation,
                transform.rotation,
                Dir3::NEG_Y,
                &ShapeCastConfig {
                    max_distance: 0.1,
                    ..default()
                },
                &SpatialQueryFilter::default()
                    .with_excluded_entities([player_entity]),
            )
            .is_some();
        if player.on_ground != on_ground {
            player.on_ground = on_ground;
        }

        if on_ground {
            if player_velocity.y <= 0.0 {
                player_velocity.y = 0.0;
            }
        }
    }
}
