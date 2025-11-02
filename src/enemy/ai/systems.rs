use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{AgentDesiredVelocity3d, AgentState, AgentTarget3d};

use crate::{
    character_controller::messages::{MovementAction, MovementDirection},
    enemy::{
        Enemy, EnemyState, shooting::components::EnemyBullet,
        spawn::AgentEnemyEntityPointer,
    },
    player::Player,
};

/// This system iterates over each enemy, and with a raycast, determines whether the enemy can see
/// the player. If yes, the enemy transform will be updated so that it looks at the player
/// transform. In addition, if the state hasn't been `AttackPlayer` yet, it will be set to
/// `AttackPlayer`. If not, the enemy state will be set to `ChasingPlayer`, if not yet set.
pub fn check_if_enemy_can_see_player(
    enemy_query: Query<(&mut Enemy, Entity, &mut Transform), Without<Player>>,
    mut enemy_agents_query: Query<(
        &AgentEnemyEntityPointer,
        &mut AgentTarget3d,
    )>,
    spatial_query: SpatialQuery,
    player_query: Single<(Entity, &Transform), With<Player>>,
    enemy_bullets: Query<Entity, With<EnemyBullet>>,
) {
    let (player_entity, player_transform) = *player_query;
    for (mut enemy, enemy_entity, mut enemy_transform) in enemy_query {
        if enemy.state == EnemyState::Dead {
            continue;
        }

        // check if we can see the player
        // direction towards player
        let vector_not_normalized =
            player_transform.translation - enemy_transform.translation;
        let Ok(direction_normalized) = Dir3::new(vector_not_normalized) else {
            continue;
        };

        let max_distance = 100.0;
        let solid = false;

        let enemy_bullet_entities: Vec<Entity> = enemy_bullets.iter().collect();

        // raycast shouldnt hit enemy itself and enemy bullets
        let filter = SpatialQueryFilter::default().with_excluded_entities(
            [[enemy_entity].to_vec(), enemy_bullet_entities].concat(),
        );

        if let Some(first_hit) = spatial_query.cast_ray(
            enemy_transform.translation,
            direction_normalized,
            max_distance,
            solid,
            &filter,
        ) {
            let enemy_can_see_player = first_hit.entity == player_entity;
            if enemy_can_see_player {
                enemy_transform.look_at(player_transform.translation, Vec3::Y);
                if enemy.state != EnemyState::AttackPlayer {
                    debug!(
                        "Enemy can see player, changing state to \
                         AttackPlayer. Previous enemy state: {:?}",
                        enemy.state
                    );
                    enemy.state = EnemyState::AttackPlayer;
                };
            } else if enemy.state != EnemyState::ChasingPlayer {
                debug!(
                    "Enemy can NOT see player, setting state to ChasingPlayer!"
                );
                let Some((_, mut agent_target)) = enemy_agents_query
                    .iter_mut()
                    .find(|(pointer, _)| pointer.0 == enemy_entity)
                else {
                    warn!(
                        "Can not update enemy agent, unable to find Enemy \
                         Agent for current entity {} via \
                         AgentEnemyEntityPointer",
                        enemy_entity
                    );
                    continue;
                };
                debug!("updating agent target to current player location");

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
                        "Could not get a valid new agent target point for \
                         chasing enemy"
                    );
                    continue;
                };

                let hit_point =
                    ray_cast_origin + first_hit.distance * ray_cast_direction;

                *agent_target = AgentTarget3d::Point(hit_point);
                enemy.state = EnemyState::ChasingPlayer;
            }
        }
    }
}

pub fn check_if_enemy_reached_target(
    mut enemy_query: Query<&mut Enemy>,
    enemy_agents_query: Query<(&AgentEnemyEntityPointer, &AgentState)>,
) {
    for (agent_enemy_entity_pointer, agent_state) in enemy_agents_query {
        let Ok(mut enemy) = enemy_query.get_mut(agent_enemy_entity_pointer.0)
        else {
            warn!(
                "Failed to find the enemy {} for given \
                 AgentPathfindingEnemyEntityPointer",
                agent_enemy_entity_pointer.0
            );
            continue;
        };
        if *agent_state == AgentState::ReachedTarget
            && enemy.state != EnemyState::Idle
        {
            enemy.state = EnemyState::CheckIfPlayerSeeable;
        }
    }
}

pub fn handle_chasing_enemies(
    mut enemy_query: Query<(&Enemy, Entity)>,
    enemy_agents_query: Query<(
        &AgentDesiredVelocity3d,
        &AgentEnemyEntityPointer,
    )>,
    mut movement_action_writer: MessageWriter<MovementAction>,
) {
    for (agent_desired_velocity, agent_enemy_entity_pointer) in
        enemy_agents_query
    {
        if agent_desired_velocity.velocity() == Vec3::ZERO {
            continue;
        }

        let Ok((enemy, entity)) =
            enemy_query.get_mut(agent_enemy_entity_pointer.0)
        else {
            warn!(
                "Failed to find the enemy {} with linearvelocity from \
                 AgentPathfindingEnemyEntityPointer",
                agent_enemy_entity_pointer.0
            );
            continue;
        };

        if enemy.state != EnemyState::ChasingPlayer {
            continue;
        }

        movement_action_writer.write(MovementAction {
            direction: MovementDirection::Move(
                agent_desired_velocity.velocity(),
            ),
            character_controller_entity: entity,
        });
    }
}
