use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{
    AgentDesiredVelocity3d, AgentState, AgentTarget3d, Velocity3d,
};
use shared::{
    character_controller::messages::{MovementAction, MovementDirection},
    enemy::{
        ENEMY_FOV, ENEMY_VISION_RANGE,
        components::{Enemy, EnemyLastStateUpdate, EnemyState},
    },
    player::Player,
    utils::transform::is_facing_target_without_y,
};

use crate::enemy::{
    ai::messages::UpdateEnemyAgentTargetMessage, spawn::AgentEnemyEntityPointer,
};

/// This system is the only system allowed to change the enemy state
/// Depending on the EnemyState, different systems will be run.
pub fn enemy_state_decision_system(
    enemy_query: Query<
        (
            Entity,
            &Transform,
            &mut EnemyState,
            &mut EnemyLastStateUpdate,
        ),
        (With<Enemy>, Without<Player>),
    >,
    player_query: Single<(Entity, &Transform), With<Player>>,
    spatial_query: SpatialQuery,
    mut set_new_enemy_agent_message_writer: MessageWriter<
        UpdateEnemyAgentTargetMessage,
    >,
) {
    for (
        enemy_entity,
        enemy_transform,
        mut enemy_state,
        mut enemy_last_state_update,
    ) in enemy_query
    {
        if enemy_last_state_update.0.elapsed().as_millis() < 1000 {
            continue;
        }

        if *enemy_state == EnemyState::Dead {
            continue;
        }
        let (player_entity, player_transform) = *player_query;

        let player_position = player_transform.translation;
        let enemy_position = enemy_transform.translation;

        // first do cheap math based check if the player is even in the fov radius of the enemy
        let to_player = player_position - enemy_position;
        let distance_to_player = to_player.length();

        let player_in_range = distance_to_player < ENEMY_VISION_RANGE;

        if player_in_range {
            // we need to check if player is in radius of current enemy -> ENEMY_FOV
            let enemy_forward = enemy_transform.forward();
            let direction = to_player.normalize();
            let angle_in_radians = enemy_forward.dot(direction);

            let player_in_enemy_fov = angle_in_radians < ENEMY_FOV.to_radians();

            if player_in_enemy_fov {
                let Ok(direction_normalized) = Dir3::new(to_player) else {
                    continue;
                };

                let max_distance = 100.0;
                let solid = false;

                // // raycast shouldnt hit enemy itself and enemy bullets
                let filter = SpatialQueryFilter::default()
                    .with_excluded_entities([enemy_entity]);

                if let Some(first_hit) = spatial_query.cast_ray(
                    enemy_transform.translation,
                    direction_normalized,
                    max_distance,
                    solid,
                    &filter,
                ) {
                    let enemy_can_see_player =
                        first_hit.entity == player_entity;
                    if enemy_can_see_player {
                        if is_facing_target_without_y(
                            enemy_transform,
                            player_transform,
                        ) {
                            enemy_state.update_state(
                                EnemyState::AttackPlayer,
                                &mut enemy_last_state_update,
                            );
                        } else {
                            enemy_state.update_state(
                                EnemyState::RotateTowardsPlayer,
                                &mut enemy_last_state_update,
                            );
                        }
                    } else if *enemy_state != EnemyState::GoToAgentTarget {
                        // the player is in range of the enemy, also in the fov cone of the enemy, but
                        // there is a obstacle in the way, so we need to give the enemy agent a new
                        // location to go to
                        set_new_enemy_agent_message_writer
                            .write(UpdateEnemyAgentTargetMessage(enemy_entity));
                        enemy_state.update_state(
                            EnemyState::GoToAgentTarget,
                            &mut enemy_last_state_update,
                        );
                    }
                }
            } else {
                info!("player is not in enemy FOV");
                enemy_state.update_state(
                    EnemyState::RotateTowardsPlayer,
                    &mut enemy_last_state_update,
                );
            }
        } else if *enemy_state != EnemyState::GoToAgentTarget {
            info!("Player is not in range of enemy, > 30m");
            enemy_state.update_state(
                EnemyState::GoToAgentTarget,
                &mut enemy_last_state_update,
            );
            set_new_enemy_agent_message_writer
                .write(UpdateEnemyAgentTargetMessage(enemy_entity));
            continue;
        }
    }
}

pub fn handle_set_new_enemy_agent_target_message(
    mut message_reader: MessageReader<UpdateEnemyAgentTargetMessage>,
    mut enemy_agents_query: Query<(
        &AgentEnemyEntityPointer,
        &mut AgentTarget3d,
    )>,
    player_query: Single<(Entity, &Transform), With<Player>>,
    spatial_query: SpatialQuery,
) {
    for message in message_reader.read() {
        let enemy_entity = message.0;
        let Some((_, mut agent_target)) = enemy_agents_query
            .iter_mut()
            .find(|(pointer, _)| pointer.0 == enemy_entity)
        else {
            warn!(
                "Can not update enemy agent, unable to find Enemy Agent for \
                 current entity {} via AgentEnemyEntityPointer",
                enemy_entity
            );
            continue;
        };
        let (player_entity, player_transform) = *player_query;

        // We use a raycast downwards, and use the hitpoint.
        // This way, it wont break if the player is mid-air, such as during a jump.
        let ray_cast_origin = player_transform.translation;
        let ray_cast_direction = Dir3::NEG_Y;

        let Some(first_hit) = spatial_query.cast_ray(
            ray_cast_origin,
            ray_cast_direction,
            10.0,
            false,
            &SpatialQueryFilter::default()
                .with_excluded_entities([player_entity]),
        ) else {
            error!(
                "Could not get a valid new agent target point for chasing \
                 enemy"
            );
            continue;
        };

        let hit_point =
            ray_cast_origin + first_hit.distance * ray_cast_direction;

        *agent_target = AgentTarget3d::Point(hit_point);

        info!("Enemy {} has a new agent target", enemy_entity);
    }
}

pub fn check_if_enemy_agent_reached_target(
    mut enemy_query: Query<(&mut EnemyState, &mut EnemyLastStateUpdate)>,
    enemy_agents_query: Query<(&AgentEnemyEntityPointer, &AgentState)>,
) {
    for (agent_enemy_entity_pointer, agent_state) in enemy_agents_query {
        if *agent_state != AgentState::ReachedTarget {
            continue;
        }

        let Ok((mut enemy_state, mut enemy_last_state_update)) =
            enemy_query.get_mut(agent_enemy_entity_pointer.0)
        else {
            warn!(
                "Failed to find the enemy {} for given \
                 AgentPathfindingEnemyEntityPointer",
                agent_enemy_entity_pointer.0
            );
            continue;
        };

        info!(
            "Enemy has reached agent target! Now checking whether the player \
             is seeable"
        );
        enemy_state.update_state(
            EnemyState::EnemyAgentReachedTarget,
            &mut enemy_last_state_update,
        );
    }
}

pub fn handle_chasing_enemies(
    mut enemy_query: Query<(&EnemyState, Entity)>,
    enemy_agents_query: Query<(
        &AgentDesiredVelocity3d,
        &AgentEnemyEntityPointer,
    )>,
    mut movement_action_writer: MessageWriter<MovementAction>,
) {
    for (agent_desired_velocity, agent_enemy_entity_pointer) in
        enemy_agents_query
    {
        let Ok((enemy_state, entity)) =
            enemy_query.get_mut(agent_enemy_entity_pointer.0)
        else {
            warn!(
                "Failed to find the enemy {} with linearvelocity from \
                 AgentPathfindingEnemyEntityPointer",
                agent_enemy_entity_pointer.0
            );
            continue;
        };

        if *enemy_state != EnemyState::GoToAgentTarget {
            continue;
        }

        // FIXME: this is problematic. character controller only runs on client.
        // this works for now as we only have enemies in singleplayer, but if we start having Waves
        // game mode multiplayer, the character controller for enemies should run on the server and
        // position be synced to all clients
        movement_action_writer.write(MovementAction {
            direction: MovementDirection::Move(
                agent_desired_velocity.velocity(),
            ),
            character_controller_entity: entity,
        });
    }
}

pub fn rotate_enemies_towards_player_over_time(
    enemy_query: Query<
        (&EnemyState, &mut Transform),
        (With<Enemy>, Without<Player>),
    >,
    player_transform: Single<&Transform, With<Player>>,
    time: Res<Time>,
) {
    for (enemy_state, mut enemy_transform) in enemy_query {
        if *enemy_state != EnemyState::RotateTowardsPlayer {
            continue;
        };

        let to_player =
            player_transform.translation - enemy_transform.translation;

        // drop y to avoid pitch rotation
        let to_player_xz = Vec3::new(to_player.x, 0.0, to_player.z).normalize();

        let enemy_forward = enemy_transform.forward();

        // drop y to avoid pitch rotation
        let forward_xz =
            Vec3::new(enemy_forward.x, 0.0, enemy_forward.z).normalize();

        let angle = forward_xz
            .cross(to_player_xz)
            .y
            .atan2(forward_xz.dot(to_player_xz));

        let target_rotation = Quat::from_rotation_y(angle);

        const ROTATION_SPEED: f32 = 6.5;

        enemy_transform.rotation = enemy_transform.rotation.slerp(
            target_rotation * enemy_transform.rotation,
            ROTATION_SPEED * time.delta_secs(),
        );
    }
}

pub fn update_enemy_agents_velocity_from_physics_velocity(
    mut agent_query: Query<(
        &mut Velocity3d,
        &AgentState,
        &AgentEnemyEntityPointer,
    )>,
    mut enemy_query: Query<(Entity, &LinearVelocity)>,
    mut message_writer: MessageWriter<UpdateEnemyAgentTargetMessage>,
) {
    for (mut agent_velocity, agent_state, agent_enemy_entity_pointer) in
        agent_query.iter_mut()
    {
        let Ok((enemy_entity, enemy_velocity)) =
            enemy_query.get_mut(agent_enemy_entity_pointer.0)
        else {
            warn!(
                "Couldn't find enemy with LinearVelocity by id {}",
                agent_enemy_entity_pointer.0
            );
            continue;
        };
        if *agent_state == AgentState::TargetNotOnNavMesh {
            // FIXME: if the player is somewhere the enemy cant reach,
            // the enemy will never be able to get to the player.
            // we should just try nearby locations instead

            // if the target is not on the navmesh, we let our systems make a new Target, until it
            // is on the navmesh again.
            message_writer.write(UpdateEnemyAgentTargetMessage(enemy_entity));
        }

        agent_velocity.velocity = enemy_velocity.0;
    }
}

// pub fn enemy_face_velocity_direction(
//     mut query: Query<(&LinearVelocity, &mut Transform, &Enemy), With<Enemy>>,
//     time: Res<Time>,
// ) {
//     for (velocity, mut transform, enemy) in &mut query {
//         continue;
//         if enemy.state == EnemyState::RotateTowardsPlayer {
//             continue;
//         }
//
//         let vel = velocity.0;
//
//         // Don’t rotate if velocity is extremely small
//         if vel.length_squared() < 0.0001 {
//             continue;
//         }
//
//         let mut dir = vel;
//         dir.y = 0.0;
//
//         if dir.length_squared() < 0.0001 {
//             continue;
//         }
//
//         // Calculate direction the enemy should face
//         let target_dir = dir.normalize();
//
//         // Create a quaternion facing that direction (Y-up)
//         let target_rotation = Quat::from_rotation_arc(Vec3::X, target_dir); // or Vec3::X depending on your model
//
//         // Smooth rotation (slerp)
//         let rotate_speed = 6.0; // adjust to taste
//         transform.rotation = transform
//             .rotation
//             .slerp(target_rotation, rotate_speed * time.delta_secs());
//     }
// }
