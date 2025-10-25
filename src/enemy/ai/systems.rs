use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{AgentState, AgentTarget3d, Velocity3d};

use crate::{
    GRAVITY,
    enemy::{
        Enemy, ai::EnemyState, shooting::components::EnemyBullet,
        spawn::AgentEnemyEntityPointer,
    },
    game_flow::states::InGameState,
    player::{
        Player,
        spawn::{PLAYER_CAPSULE_LENGTH, PLAYER_CAPSULE_RADIUS},
    },
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
                    info!(
                        "Enemy can see player, changing state to AttackPlayer"
                    );
                    enemy.state = EnemyState::AttackPlayer;
                };
            } else {
                if enemy.state != EnemyState::ChasingPlayer {
                    info!(
                        "Enemy can NOT see player, setting state to \
                         ChasingPlayer!"
                    );
                    enemy.state = EnemyState::ChasingPlayer;
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
                    info!("updating agent target to current playerr location");
                    *agent_target = AgentTarget3d::Point(
                        player_transform.translation.with_y(0.),
                    );
                }
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
            info!("Enemy reached target, setting state to Idle");
            enemy.state = EnemyState::CheckIfPlayerSeeable;
        }
    }
}

/// Ensures that if our enemy is not in ChasingPlayer state the velocity will be 0.
/// Useful so we dont have to set velocity to 0 in all systems where we mutate the enemy state
pub fn set_zero_velocity_if_not_chasing(
    enemy_query: Query<(&Enemy, &mut LinearVelocity)>,
) {
    for (enemy, mut velocity) in enemy_query {
        if enemy.state != EnemyState::ChasingPlayer && velocity.0 != Vec3::ZERO
        {
            info!("current velocity: {}", velocity.0);
            info!("Enemy no longer chasing player, zeoring velocity!");
            velocity.0 = Vec3::ZERO;
        }
    }
}

pub fn handle_chasing_enemies(
    mut enemy_query: Query<(&Enemy, &mut LinearVelocity)>,
    enemy_agents_query: Query<(&Velocity3d, &AgentEnemyEntityPointer)>,
    current_in_game_state: Res<State<InGameState>>,
) {
    for (agent_velocity, agent_enemy_entity_pointer) in enemy_agents_query {
        let Ok((enemy, mut velocity)) =
            enemy_query.get_mut(agent_enemy_entity_pointer.0)
        else {
            warn!(
                "Failed to find the enemy {} with linearvelocity from \
                 AgentPathfindingEnemyEntityPointer",
                agent_enemy_entity_pointer.0
            );
            continue;
        };

        let in_game_state_is_playing =
            *current_in_game_state.get() == InGameState::Playing;
        if !in_game_state_is_playing {
            **velocity = Vec3::ZERO;
            continue;
        }

        if enemy.state != EnemyState::ChasingPlayer {
            continue;
        }

        velocity.0 = agent_velocity.velocity;
    }
}

pub fn update_enemy_on_ground(
    enemies: Query<(&mut Enemy, &Transform, Entity, &mut LinearVelocity)>,
    spatial_query: SpatialQuery,
) {
    for (mut enemy, transform, player_entity, mut player_velocity) in enemies {
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
        if enemy.on_ground != on_ground {
            enemy.on_ground = on_ground;
        }

        if on_ground {
            if player_velocity.y <= 0.0 {
                player_velocity.y = 0.0;
            }
        }
    }
}

pub fn apply_gravity_over_time(
    mut enemy_query: Single<(&Enemy, &mut LinearVelocity)>,
    time: Res<Time>,
) {
    let enemy = enemy_query.0;
    let enemy_velocity = &mut enemy_query.1;

    if !enemy.on_ground && enemy_velocity.y > 0.0 {
        enemy_velocity.y -= GRAVITY * time.delta_secs();
    }
}
