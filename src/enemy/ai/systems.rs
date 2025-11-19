use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{AgentDesiredVelocity3d, AgentState, AgentTarget3d};

use crate::{
    character_controller::messages::{MovementAction, MovementDirection},
    enemy::{
        Enemy, EnemyState, ai::ENEMY_VISION_RANGE,
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
) {
    let (player_entity, player_transform) = *player_query;
    let player_position = player_transform.translation;

    for (mut enemy, enemy_entity, mut enemy_transform) in enemy_query {
        if enemy.state == EnemyState::Dead {
            continue;
        }

        // first do cheap math based check if the player is even in the fov radius of the enemy
        let enemy_position = enemy_transform.translation;

        let to_player = player_position - enemy_position;
        let distance = to_player.length();

        if distance > ENEMY_VISION_RANGE {
            enemy.state = EnemyState::GoToLastKnownLocation;
            continue;
        }

        let enemy_forward = enemy_transform.forward();
        let direction = to_player.normalize();
        let angle = enemy_forward.dot(direction);

        if angle < 0.5 {
            enemy.state = EnemyState::GoToLastKnownLocation;
            continue;
        }

        // if player is in the fov radius, we do a raycast from the enemy towards the players
        // current position
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
            } else if enemy.state != EnemyState::GoToLastKnownLocation {
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
                enemy.state = EnemyState::GoToLastKnownLocation;
            }
        }
    }
}

pub fn check_if_enemy_reached_target(
    mut enemy_query: Query<&mut Enemy>,
    enemy_agents_query: Query<(&AgentEnemyEntityPointer, &AgentState)>,
) {
    for (agent_enemy_entity_pointer, agent_state) in enemy_agents_query {
        if *agent_state != AgentState::ReachedTarget {
            continue;
        }

        let Ok(mut enemy) = enemy_query.get_mut(agent_enemy_entity_pointer.0)
        else {
            warn!(
                "Failed to find the enemy {} for given \
                 AgentPathfindingEnemyEntityPointer",
                agent_enemy_entity_pointer.0
            );
            continue;
        };
        enemy.state = EnemyState::CheckIfPlayerSeeable;
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

        if enemy.state != EnemyState::GoToLastKnownLocation {
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

fn get_closest_transform_from_point(
    source: Vec3,
    targets: Vec<Vec3>,
) -> Option<Vec3> {
    if targets.is_empty() {
        return None;
    }

    let mut closest_point = targets[0];

    for point in targets {
        let distance_of_closest_point = closest_point.length();

        let distance = (source - point).length();
        if distance_of_closest_point < distance {
            closest_point = point;
        }
    }

    Some(closest_point)
}
