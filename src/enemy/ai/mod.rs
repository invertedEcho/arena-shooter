use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{enemy::Enemy, game_flow::GameState, player::Player};

// TODO: actually implement this lol
// Enemy AI is currrently working like this:
// A raycast is shot to the direction of the player
//    If the raycast first hit is the player, the enemy state changes to AttackPlayer and will
//    shoot him.
//    If not, it means the enemy can not see the player. Then, we use the pathfinding library
//    together with our navmesh to find the fastest path to the player
//    (in the future we will of course not just take the fastest route, but have some kind of
//    randomness?)

pub struct EnemyAiPlugin;

impl Plugin for EnemyAiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CheckIfEnemyCanSeePlayerCooldownTimer(
            Timer::from_seconds(0.1, TimerMode::Repeating),
        ))
        .add_systems(
            Update,
            (
                check_if_enemy_can_see_player_and_look_at_player,
                tick_enemy_can_see_player_cooldown_timer,
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Default, Reflect, PartialEq)]
pub enum EnemyAiState {
    #[default]
    Idle,
    /// Going to the last known location of player
    ChasingPlayer,
    /// Enemy can see the player, will shoot the player now
    AttackPlayer,
}

#[derive(Resource)]
pub struct CheckIfEnemyCanSeePlayerCooldownTimer(pub Timer);

fn tick_enemy_can_see_player_cooldown_timer(
    mut timer: ResMut<CheckIfEnemyCanSeePlayerCooldownTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());
}

fn check_if_enemy_can_see_player_and_look_at_player(
    spatial_query: SpatialQuery,
    enemy_query: Query<
        (&mut Enemy, Entity, &mut Transform),
        (Without<Player>, With<Enemy>),
    >,
    player_query: Single<(Entity, &Transform), With<Player>>,
    timer: Res<CheckIfEnemyCanSeePlayerCooldownTimer>,
) {
    if timer.0.just_finished() {
        let player_entity = player_query.0;
        let player_transform = player_query.1;

        for (mut enemy, enemy_entity, mut enemy_transform) in enemy_query {
            let enemy_translation = enemy_transform.translation;
            let player_translation = player_transform.translation;

            let origin = enemy_translation;

            // direction towards player
            let vector_not_normalized = Vec3 {
                x: player_translation.x - enemy_translation.x,
                y: player_translation.y - enemy_translation.y,
                z: player_translation.z - enemy_translation.z,
            };
            let direction = Dir3::new(vector_not_normalized).unwrap();

            let max_distance = 1000.0;
            let solid = false;

            // raycast shouldnt hit enemy itself
            let filter = SpatialQueryFilter::default()
                .with_excluded_entities([enemy_entity]);

            if let Some(first_hit) = spatial_query.cast_ray(
                origin,
                direction,
                max_distance,
                solid,
                &filter,
            ) {
                if first_hit.entity == player_entity {
                    if enemy.state != EnemyAiState::AttackPlayer {
                        enemy.state = EnemyAiState::AttackPlayer;
                    }
                    enemy_transform
                        .look_at(player_transform.translation, Dir3::Y);
                }
            }
        }
    }
}
