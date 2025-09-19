use std::ops::Neg;

use avian3d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};

use crate::{
    common::{BULLET_VELOCITY, components::DespawnTimer},
    enemy::{animate::AnimateEnemyPlugin, spawn::EnemySpawnPlugin},
    game_flow::GameState,
    player::{Player, shooting::components::PlayerBullet},
};

mod animate;
mod spawn;

const SWAT_MODEL_PATH: &str = "models/animated/SWAT.glb";

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnemySpawnPlugin)
            .add_plugins(AnimateEnemyPlugin)
            .register_type::<Enemy>()
            .register_type::<EnemySpawnLocation>()
            .insert_resource(CheckIfEnemyCanSeePlayerCooldownTimer(
                Timer::from_seconds(0.1, TimerMode::Repeating),
            ))
            .add_systems(
                Update,
                (
                    check_if_enemy_can_see_player_and_look_at_player,
                    tick_enemy_can_see_player_cooldown_timer,
                    enemy_shoot_player,
                    tick_enemy_shoot_player_cooldown_timer,
                    detect_player_bullet_collision_with_enemy,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Enemy {
    state: EnemyState,
    pub health: f32,
}

#[derive(Default, Reflect, PartialEq)]
enum EnemyState {
    #[default]
    SearchForPlayer,
    ChasingPlayer,
    AttackPlayer,
}

#[derive(Resource)]
pub struct CheckIfEnemyCanSeePlayerCooldownTimer(pub Timer);

#[derive(Component)]
pub struct EnemyBullet;

#[derive(Component)]
pub struct EnemyShootPlayerCooldownTimer(pub Timer);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EnemySpawnLocation;

fn detect_player_bullet_collision_with_enemy(
    mut commands: Commands,
    player_bullet_query: Query<(Entity, &PlayerBullet)>,
    mut enemy_query: Query<(Entity, &mut Enemy)>,
    mut collision_event_reader: EventReader<CollisionStarted>,
) {
    for CollisionStarted(first_entity, second_entity) in
        collision_event_reader.read()
    {
        let Some(player_bullet) =
            player_bullet_query.iter().find(|(entity, _)| {
                entity == first_entity || entity == second_entity
            })
        else {
            continue;
        };

        let Some(mut enemy) = enemy_query.iter_mut().find(|(entity, _)| {
            entity == first_entity || entity == second_entity
        }) else {
            continue;
        };

        enemy.1.health -= player_bullet.1.damage;
        if enemy.1.health <= 0.0 {
            commands.entity(enemy.0).despawn();
        }
    }
}

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

            let max_distance = 20.0;
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
                    // info!(
                    //     "Enemy {} can see player, changing state to attack and looking at player!",
                    //     enemy_entity
                    // );
                    enemy.state = EnemyState::AttackPlayer;
                    enemy_transform
                        .look_at(player_transform.translation, Dir3::Y);
                } else {
                    // info!(
                    //     "first hit was not player, but something else: {} own id: {}",
                    //     first_hit.entity, enemy_entity
                    // );

                    enemy.state = EnemyState::SearchForPlayer;
                }
            }
        }
    }
}

fn enemy_shoot_player(
    mut commands: Commands,
    enemy_query: Query<(&Enemy, &Transform, &EnemyShootPlayerCooldownTimer)>,
) {
    for (enemy, enemy_transform, enemy_shoot_player_cooldown_timer) in
        enemy_query
    {
        if enemy.state == EnemyState::AttackPlayer
            && enemy_shoot_player_cooldown_timer.0.finished()
        {
            let local_bullet_velocity = Vec3 {
                z: BULLET_VELOCITY.neg(),
                x: 0.0,
                y: 0.0,
            };
            let world_bullet_velocity =
                enemy_transform.rotation * local_bullet_velocity;

            commands.spawn((
                Transform {
                    translation: Vec3 {
                        x: enemy_transform.translation.x,
                        y: enemy_transform.translation.y,
                        z: enemy_transform.translation.z,
                    },
                    ..default()
                },
                Collider::cuboid(0.1, 0.1, 0.1),
                Sensor,
                LinearVelocity(world_bullet_velocity),
                RigidBody::Kinematic,
                DespawnTimer(Timer::from_seconds(3.0, TimerMode::Once)),
                EnemyBullet,
            ));
        }
    }
}

fn tick_enemy_shoot_player_cooldown_timer(
    timer_query: Query<&mut EnemyShootPlayerCooldownTimer>,
    time: Res<Time>,
) {
    for mut timer in timer_query {
        timer.0.tick(time.delta());
    }
}
