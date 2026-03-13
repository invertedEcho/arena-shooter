use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{
    AgentDesiredVelocity3d, AgentState, AgentTarget3d, Velocity3d,
};
use shared::{
    character_controller::apply_collide_and_slide,
    enemy::{
        ENEMY_FOV, ENEMY_VISION_RANGE, UPDATE_ENEMY_STATE_COOLDOWN_SECONDS,
        components::{Enemy, EnemyLastStateUpdate, EnemyState},
    },
    player::Player,
    utils::transform::is_facing_target_without_y,
    world_object::WorldObjectCollectibleServerSide,
};

use crate::enemy::{
    ai::messages::{PlayerHitEnemy, UpdateEnemyAgentTargetMessage},
    spawn::EnemyAgent,
};

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
        if enemy_last_state_update.0.elapsed().as_secs()
            < UPDATE_ENEMY_STATE_COOLDOWN_SECONDS
        {
            continue;
        }

        let enemy_is_dead = *enemy_state == EnemyState::Dead;
        if enemy_is_dead {
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
            debug!("Player is in range of enemy {}", enemy_entity);
            // we need to check if player is in radius of current enemy -> ENEMY_FOV
            let enemy_forward = enemy_transform.forward();
            let Some(direction_to_player) = to_player.try_normalize() else {
                debug!("Failed to get direction to player");
                continue;
            };
            let angle_between_enemy_and_player =
                enemy_forward.dot(direction_to_player);

            let player_in_enemy_fov =
                angle_between_enemy_and_player < ENEMY_FOV.to_radians();

            if player_in_enemy_fov {
                debug!("Player is in enemy FOV. Enemy {}", enemy_entity);
                let Ok(direction_to_player_as_dir) =
                    Dir3::new(direction_to_player)
                else {
                    continue;
                };

                let max_distance = 100.0;
                let solid = false;

                // raycast shouldnt hit enemy itself
                let filter = SpatialQueryFilter::default()
                    .with_excluded_entities([enemy_entity]);

                let ray_cast_result = spatial_query.cast_ray(
                    enemy_transform.translation,
                    direction_to_player_as_dir,
                    max_distance,
                    solid,
                    &filter,
                );

                if let Some(first_hit) = ray_cast_result {
                    let enemy_can_see_player =
                        first_hit.entity == player_entity;
                    if enemy_can_see_player {
                        debug!("Enemy {} can see the player", enemy_entity);
                        if is_facing_target_without_y(
                            enemy_transform,
                            player_transform,
                        ) {
                            enemy_state.update_state(
                                EnemyState::AttackPlayer(player_entity),
                                &mut enemy_last_state_update,
                                false,
                            );
                        } else {
                            enemy_state.update_state(
                                EnemyState::RotateTowardsPlayer(player_entity),
                                &mut enemy_last_state_update,
                                false,
                            );
                        }
                    } else if *enemy_state == EnemyState::GoToAgentTarget {
                        debug!(
                            "Enemy {} cant see the player but is still in \
                             GoToAgentTarget state",
                            enemy_entity
                        );
                    } else if *enemy_state != EnemyState::GoToAgentTarget {
                        debug!(
                            "Enemy can't see the player, going to agent target"
                        );
                        // the player is in range of the enemy, also in the fov cone of the enemy, but
                        // there is a obstacle in the way, so we need to give the enemy agent a new
                        // location to go to
                        set_new_enemy_agent_message_writer
                            .write(UpdateEnemyAgentTargetMessage(enemy_entity));
                        enemy_state.update_state(
                            EnemyState::GoToAgentTarget,
                            &mut enemy_last_state_update,
                            false,
                        );
                    }
                }
            } else {
                enemy_state.update_state(
                    EnemyState::RotateTowardsPlayer(player_entity),
                    &mut enemy_last_state_update,
                    false,
                );
            }
        } else if *enemy_state != EnemyState::GoToAgentTarget {
            debug!("Player is not in range of enemy {}", enemy_entity);
            enemy_state.update_state(
                EnemyState::GoToAgentTarget,
                &mut enemy_last_state_update,
                false,
            );
            set_new_enemy_agent_message_writer
                .write(UpdateEnemyAgentTargetMessage(enemy_entity));
        }
    }
}

pub fn handle_set_new_enemy_agent_target_message(
    mut commands: Commands,
    mut message_reader: MessageReader<UpdateEnemyAgentTargetMessage>,
    mut enemy_agents_query: Query<&mut AgentTarget3d>,
    enemy_query: Query<(&EnemyState, &EnemyAgent), With<Enemy>>,
    player_query: Single<(Entity, &Transform), With<Player>>,
    spatial_query: SpatialQuery,
) {
    for message in message_reader.read() {
        let enemy_entity = message.0;

        let Ok((enemy_state, enemy_agent)) = enemy_query.get(enemy_entity)
        else {
            continue;
        };

        if enemy_state.is_dead() {
            continue;
        }

        let Ok(mut agent_target) = enemy_agents_query.get_mut(enemy_agent.0)
        else {
            warn!(
                "Can not update enemy agent, unable to find Enemy Agent {} \
                 for enemy entity {}",
                enemy_agent.0, enemy_entity
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
            info!(
                "Could not get a valid new agent target point for chasing \
                 enemy {}, inserting RetryGetNewAgentTarget timer",
                enemy_entity
            );
            commands.entity(enemy_entity).insert(RetryGetNewAgentTarget(
                Timer::from_seconds(1.0, TimerMode::Once),
            ));
            continue;
        };

        let hit_point =
            ray_cast_origin + first_hit.distance * ray_cast_direction;

        *agent_target = AgentTarget3d::Point(hit_point);

        debug!("Enemy {} has a new agent target", enemy_entity);
    }
}

/// This component gets inserted into enemies which failed to get a new agent target. After the
/// timer has finished, we write UpdateEnemyAgentTargetMessage to try again.
/// Failures of getting a valid agent target may be for example: the player is currently not on
/// the nav mesh
#[derive(Component)]
pub struct RetryGetNewAgentTarget(pub Timer);

pub fn retry_get_new_agent_target(
    mut commands: Commands,
    query: Query<(Entity, &mut RetryGetNewAgentTarget), With<Enemy>>,
    time: Res<Time>,
    mut update_enemy_target_message_writer: MessageWriter<
        UpdateEnemyAgentTargetMessage,
    >,
) {
    for (enemy_entity, mut timer) in query {
        timer.0.tick(time.delta());
        if timer.0.is_finished() {
            update_enemy_target_message_writer
                .write(UpdateEnemyAgentTargetMessage(enemy_entity));
            info!(
                "RetryGetNewAgentTarget timer has finished, enemy entity {} \
                 will now try to get new agent target",
                enemy_entity
            );
            commands
                .entity(enemy_entity)
                .remove::<RetryGetNewAgentTarget>();
        }
    }
}

pub fn check_if_enemy_agent_reached_target(
    mut enemy_query: Query<(
        &mut EnemyState,
        &mut EnemyLastStateUpdate,
        &mut LinearVelocity,
    )>,
    enemy_agents_query: Query<(&AgentState, &mut AgentTarget3d, &ChildOf)>,
) {
    for (agent_state, mut agent_target, enemy_parent) in enemy_agents_query {
        if *agent_state != AgentState::ReachedTarget {
            continue;
        }

        let Ok((mut enemy_state, mut enemy_last_state_update, mut velocity)) =
            enemy_query.get_mut(enemy_parent.0)
        else {
            continue;
        };

        debug!(
            "Enemy has reached agent target! Now checking whether the player \
             is seeable"
        );
        enemy_state.update_state(
            EnemyState::EnemyAgentReachedTarget,
            &mut enemy_last_state_update,
            false,
        );
        velocity.0 = Vec3::ZERO;
        *agent_target = AgentTarget3d::None;
    }
}

pub fn handle_chasing_enemies(
    mut enemy_query: Query<(
        &EnemyState,
        Entity,
        &mut LinearVelocity,
        &Transform,
    )>,
    enemy_agents_query: Query<(&AgentDesiredVelocity3d, &ChildOf)>,
    mut spatial_query: SpatialQuery,
    world_object_collectible: Query<
        Entity,
        With<WorldObjectCollectibleServerSide>,
    >,
    time: Res<Time>,
) {
    for (agent_desired_velocity, enemy_parent) in enemy_agents_query {
        let Ok((enemy_state, entity, mut velocity, transform)) =
            enemy_query.get_mut(enemy_parent.0)
        else {
            continue;
        };

        if *enemy_state != EnemyState::GoToAgentTarget {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        // we dont apply y from agent velocity as otherwise it would fight our gravity system
        velocity.x = agent_desired_velocity.velocity().x;
        velocity.z = agent_desired_velocity.velocity().z;

        let excluded_entities: Vec<Entity> = world_object_collectible
            .iter()
            .chain(std::iter::once(entity))
            .collect();

        let spatial_query_filter = &SpatialQueryFilter::default()
            .with_excluded_entities(excluded_entities.clone());

        apply_collide_and_slide(
            &mut velocity,
            agent_desired_velocity.velocity(),
            transform,
            &mut spatial_query,
            spatial_query_filter,
            time.delta_secs(),
            0,
            false,
        );
    }
}

pub fn rotate_enemies_towards_player_over_time(
    enemy_query: Query<
        (&EnemyState, &mut Transform),
        (With<Enemy>, Without<Player>),
    >,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    for (enemy_state, mut enemy_transform) in enemy_query {
        let EnemyState::RotateTowardsPlayer(player_entity) = enemy_state else {
            continue;
        };

        let Ok(player_transform) = player_query.get(*player_entity) else {
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

pub fn update_enemy_agents_velocity(
    mut agent_query: Query<(&mut Velocity3d, &AgentState, &ChildOf)>,
    mut enemy_query: Query<(Entity, &LinearVelocity)>,
    mut message_writer: MessageWriter<UpdateEnemyAgentTargetMessage>,
) {
    for (mut agent_velocity, agent_state, enemy_parent) in
        agent_query.iter_mut()
    {
        let Ok((enemy_entity, enemy_velocity)) =
            enemy_query.get_mut(enemy_parent.0)
        else {
            continue;
        };
        if *agent_state == AgentState::TargetNotOnNavMesh {
            // if the target is not on the navmesh, we let our systems make a new Target, until it
            // is on the navmesh again.
            message_writer.write(UpdateEnemyAgentTargetMessage(enemy_entity));
        }

        agent_velocity.velocity = enemy_velocity.0;
    }
}

pub fn zero_enemy_velocity(
    enemy_query: Query<&mut LinearVelocity, With<Enemy>>,
) {
    for mut enemy in enemy_query {
        enemy.0 = Vec3::ZERO;
    }
}

pub fn read_player_hit_enemy_messages(
    mut message_reader: MessageReader<PlayerHitEnemy>,
    mut enemy_query: Query<
        (&mut EnemyState, &mut EnemyLastStateUpdate),
        With<Enemy>,
    >,
) {
    for message in message_reader.read() {
        let Ok((mut enemy_state, mut enemy_last_state_update)) =
            enemy_query.get_mut(message.enemy_entity)
        else {
            continue;
        };

        if enemy_state.is_dead() {
            continue;
        }

        // first we want to rotate the enemy towards player, so it looks like the enemy noticed
        // that he was shot, and rotates towards the shot origin

        enemy_state.update_state(
            EnemyState::RotateTowardsPlayer(message.player_entity),
            &mut enemy_last_state_update,
            false,
        );
    }
}
