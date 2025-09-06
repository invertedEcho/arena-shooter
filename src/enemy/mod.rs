use avian3d::prelude::*;
use bevy::prelude::*;

use crate::player::Player;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Enemy>()
            .insert_resource(CheckIfEnemyCanSeePlayerCooldownTimer(
                Timer::from_seconds(2.0, TimerMode::Repeating),
            ))
            .add_systems(
                Update,
                (
                    check_if_enemy_can_see_player,
                    tick_enemy_can_see_player_cooldown_timer,
                    rotate_enemy_to_face_toward_player,
                ),
            );
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Enemy {
    #[reflect(default)]
    pub can_see_player: bool,
}

#[derive(Resource)]
pub struct CheckIfEnemyCanSeePlayerCooldownTimer(pub Timer);

fn tick_enemy_can_see_player_cooldown_timer(
    mut timer: ResMut<CheckIfEnemyCanSeePlayerCooldownTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());
}

fn check_if_enemy_can_see_player(
    spatial_query: SpatialQuery,
    enemy_query: Query<(&mut Enemy, Entity, &Transform)>,
    player_query: Single<(Entity, &Transform), With<Player>>,
    timer: Res<CheckIfEnemyCanSeePlayerCooldownTimer>,
) {
    if timer.0.just_finished() {
        let player_entity = player_query.0;
        let player_transform = player_query.1;

        for (mut enemy, enemy_entity, enemy_transform) in enemy_query {
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

            let max_distance = 20.0;
            let solid = true;

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
                    info!("Enemy can see the player!");
                    enemy.can_see_player = true;
                } else {
                    info!(
                        "Ray cast didnt hit player but hit: {}",
                        first_hit.entity
                    );
                }
            }
        }
    }
}

fn rotate_enemy_to_face_toward_player(
    enemy_query: Query<(&Enemy, &mut Transform, &mut AngularVelocity)>,
    player_transform: Single<&Transform, (With<Player>, Without<Enemy>)>,
) {
    for (enemy, mut enemy_transform, mut angular_velocity) in enemy_query {
        if enemy.can_see_player {
            // let destination_transform =
            enemy_transform.look_at(player_transform.translation, Dir3::Y);
        }
    }
}
