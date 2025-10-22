use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::AgentDesiredVelocity3d;

use crate::{
    GRAVITY,
    enemy::{Enemy, spawn::AgentPathfindingEnemyEntityPointer},
    game_flow::states::{AppState, InGameState},
    player::{
        Player,
        spawn::{PLAYER_CAPSULE_LENGTH, PLAYER_CAPSULE_RADIUS},
    },
};

/// Enemy AI is currrently working like this:
/// - A raycast is shot to the direction of the player
///    - If the raycast first hit is the player, the enemy state changes to AttackPlayer and will
///      shoot him.
///    - If not, it means the enemy can not see the player. Then, we use the pathfinding library
///      together with our navmesh to find the fastest path to the player
///      (in the future we will of course not just take the fastest route, but have some kind of
///      randomness?)
pub struct EnemyAiPlugin;

impl Plugin for EnemyAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<StartChasingPlayerMessage>().add_systems(
            Update,
            (
                enemy_patrol,
                handle_enemy_state_transition_to_chase_player,
                check_if_enemy_can_see_player,
                update_enemy_on_ground,
                apply_gravity_over_time,
                handle_start_chasing_player_message,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Default, Reflect, PartialEq, Debug)]
pub enum EnemyState {
    #[default]
    Idle,
    /// Going to the location of the player
    ChasingPlayer,
    /// Enemy can see the player, will shoot the player now
    AttackPlayer,
    /// This state will be set when `enemy.health == 0.0`. A death animation will be played and
    /// afterwards the enemy will be despawned.
    Dead,
}

/// This event will get fired when the enemy can not directly see the player.
#[derive(Message)]
pub struct StartChasingPlayerMessage {
    pub enemy_entity: Entity,
}

/// This system iterates over each enemy, and with a raycast, determines whether the enemy can see
/// the player. If yes, the enemy transform will be updated so that it looks at the player
/// transform. In addition, if the state hasn't been `AttackPlayer` yet, it will be set to
/// `AttackPlayer`. If not, the enemy state will be set to `ChasingPlayer`, if not yet set.
fn check_if_enemy_can_see_player(
    enemy_query: Query<(&mut Enemy, Entity, &mut Transform), Without<Player>>,
    spatial_query: SpatialQuery,
    player_query: Single<(Entity, &Transform), With<Player>>,
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
        let direction_normalized = Dir3::new(vector_not_normalized).unwrap();

        let max_distance = 100.0;
        let solid = false;

        // raycast shouldnt hit enemy itself
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
                    enemy.state = EnemyState::AttackPlayer;
                };
            } else {
                if enemy.state != EnemyState::ChasingPlayer {
                    info!(
                        "Enemy can NOT see player, setting state to \
                         ChasingPlayer!"
                    );
                    enemy.state = EnemyState::ChasingPlayer;
                }
            }
        }
    }
}

// This is a seperate system, so we only fire the event on a Change to chasing player, not
// just every frame the enemy cant see the player
fn handle_enemy_state_transition_to_chase_player(
    changed_enemies: Query<(&Enemy, Entity), Changed<Enemy>>,
    mut start_chasing_player_message_writer: MessageWriter<
        StartChasingPlayerMessage,
    >,
) {
    for (enemy, enemy_entity) in changed_enemies {
        if enemy.state == EnemyState::ChasingPlayer {
            // TODO: this feels kinda useless? we just write a message when the state changes, why
            // not directly write the message instead of changing the state
            info!(
                "Enemy {} changed state to ChasingPlayer, firing \
                 StartChasingPlayerMessage",
                enemy_entity
            );
            start_chasing_player_message_writer
                .write(StartChasingPlayerMessage { enemy_entity });
        }
    }
}

fn handle_start_chasing_player_message(
    mut start_chasing_player_message_reader: MessageReader<
        StartChasingPlayerMessage,
    >,
    mut enemy_query: Query<
        (Entity, &mut Transform),
        (Without<Player>, With<Enemy>),
    >,
) {
    for event in start_chasing_player_message_reader.read() {
        let Some((enemy_entity, mut _enemy_transform)) = enemy_query
            .iter_mut()
            .find(|(entity, _)| *entity == event.enemy_entity)
        else {
            warn!(
                "A StartChasingPlayerMessage was read, but the enemy entity \
                 from the event couldn't be found."
            );
            continue;
        };
        info!(
            "StartChasingPlayerMessage was read for enemy_entity: {}",
            enemy_entity
        );
    }
}

fn enemy_patrol(
    mut enemy_query: Query<(Entity, &Enemy, &mut LinearVelocity)>,
    enemy_agents_query: Query<(
        &AgentDesiredVelocity3d,
        &AgentPathfindingEnemyEntityPointer,
    )>,
    current_in_game_state: Res<State<InGameState>>,
) {
    for (agent_desired_velocity, enemy_entity) in enemy_agents_query {
        let Ok((entity, enemy, mut velocity)) =
            enemy_query.get_mut(enemy_entity.0)
        else {
            warn!(
                "Failed to find the enemy {} with linearvelocity from \
                 AgentPathfindingEnemyEntityPointer",
                enemy_entity.0
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

        info!(
            "updating velocity of enemy {} in chasingplayer state via \
             desired_velocity",
            entity
        );

        velocity.0 = agent_desired_velocity.velocity();
    }
}

fn update_enemy_on_ground(
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

fn apply_gravity_over_time(
    mut enemy_query: Single<(&Enemy, &mut LinearVelocity)>,
    time: Res<Time>,
) {
    let enemy = enemy_query.0;
    let enemy_velocity = &mut enemy_query.1;

    if !enemy.on_ground {
        enemy_velocity.y -= GRAVITY * time.delta_secs();
    }
}
