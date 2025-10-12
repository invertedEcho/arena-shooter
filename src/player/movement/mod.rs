use avian3d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};

use crate::{
    GRAVITY,
    common::MovementState,
    game_flow::states::{AppState, InGameState},
    player::{
        Player,
        camera::components::ViewModelCamera,
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
pub struct DebugHitPoints;

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
    )>,
    player_camera_entity: Single<Entity, With<ViewModelCamera>>,
    spatial_query: SpatialQuery,
    current_in_game_state: Res<State<InGameState>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    debug_hit_points: Query<
        &mut Transform,
        (With<DebugHitPoints>, Without<Player>),
    >,
) {
    let (
        entity,
        player,
        mut velocity,
        player_transform,
        mut player_movement_state,
    ) = player.into_inner();

    let currently_playing =
        *current_in_game_state.get() == InGameState::Playing;
    if !currently_playing {
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
        player_movement_state.0 = MovementState::Idle;
        return;
    }

    let world_velocity = player_transform.rotation * local_velocity * speed;
    let Ok(direction) = Dir3::new(world_velocity) else {
        return;
    };

    // all this origin, rotation and direction is definitely correct
    if let Some(first_hit) = spatial_query.cast_shape(
        &Collider::capsule(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_LENGTH),
        player_transform.translation - direction.as_vec3() * 0.025,
        player_transform.rotation,
        direction,
        &ShapeCastConfig {
            max_distance: 0.15,
            ..default()
        },
        &SpatialQueryFilter::default()
            .with_excluded_entities([entity, *player_camera_entity]),
    ) {
        debug!("Disallowing movement, got hit: {:?}", first_hit);
        if debug_hit_points.iter().len() == 0 {
            commands.spawn((
                Transform::from_translation(first_hit.point1),
                Mesh3d(meshes.add(Sphere::new(0.1))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: RED.into(),
                    ..Default::default()
                })),
                DebugHitPoints,
            ));
        } else {
            for mut debug_hit_point_transform in debug_hit_points {
                debug_hit_point_transform.translation = first_hit.point1;
            }
        }

        velocity.x = 0.0;
        velocity.z = 0.0;
        return;
    }

    velocity.x = world_velocity.x;
    velocity.z = world_velocity.z;

    if speed == PLAYER_RUN_VELOCITY {
        if player_movement_state.0 != MovementState::Running {
            player_movement_state.0 = MovementState::Running;
        }
    } else if local_velocity != Vec3::ZERO {
        if player_movement_state.0 != MovementState::Walking {
            player_movement_state.0 = MovementState::Walking;
        }
    } else {
        if player_movement_state.0 != MovementState::Idle {
            player_movement_state.0 = MovementState::Idle;
        }
    }
}

fn apply_gravity_over_time(
    player: Single<&Player>,
    mut player_velocity: Single<&mut LinearVelocity, With<Player>>,
    time: Res<Time>,
) {
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
