use avian3d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};

use crate::{
    common::components::DespawnTimer,
    game_flow::GameState,
    player::{Player, shooting::components::PlayerBullet},
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Enemy>()
            .insert_resource(CheckIfEnemyCanSeePlayerCooldownTimer(
                Timer::from_seconds(0.1, TimerMode::Repeating),
            ))
            .add_systems(
                Update,
                (
                    check_if_enemy_can_see_player_and_look_at_player,
                    tick_enemy_can_see_player_cooldown_timer,
                    enemy_shoot_playerr,
                    handle_enemy_shoot_player_cooldown_timer,
                    detect_player_bullet_collision_with_enemy,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Enemy {
    pub can_see_player: bool,
    pub health: f32,
}

#[derive(Resource)]
pub struct CheckIfEnemyCanSeePlayerCooldownTimer(pub Timer);

#[derive(Component)]
pub struct EnemyBullet;

#[derive(Component)]
pub struct EnemyShootPlayerCooldownTimer(pub Timer);

fn detect_player_bullet_collision_with_enemy(
    mut commands: Commands,
    player_bullet_query: Query<Entity, With<PlayerBullet>>,
    mut enemy_query: Query<(Entity, &mut Enemy)>,
    mut collision_event_reader: EventReader<CollisionStarted>,
) {
    for CollisionStarted(first_entity, second_entity) in
        collision_event_reader.read()
    {
        let is_player_bullet = player_bullet_query
            .iter()
            .any(|entity| entity == *first_entity || entity == *second_entity);
        if !is_player_bullet {
            continue;
        }
        let Some(mut enemy) = enemy_query.iter_mut().find(|(entity, _)| {
            entity == first_entity || entity == second_entity
        }) else {
            continue;
        };

        enemy.1.health -= 10.0;
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

// TODO: should probably be two systems
fn check_if_enemy_can_see_player_and_look_at_player(
    spatial_query: SpatialQuery,
    enemy_query: Query<(&mut Enemy, Entity, &mut Transform), Without<Player>>,
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
                    enemy.can_see_player = true;
                    enemy_transform
                        .look_at(player_transform.translation, Dir3::Y);
                } else {
                    enemy.can_see_player = false;
                }
            }
        }
    }
}

fn enemy_shoot_playerr(
    mut commands: Commands,
    enemy_query: Query<(&Enemy, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // TODO: This will break if multiple enemies exist... great, what a good system..
    enemy_can_shoot_player_cooldown_timer: Query<
        &EnemyShootPlayerCooldownTimer,
    >,
) {
    for (enemy, enemy_transform) in enemy_query {
        if enemy.can_see_player
            && enemy_can_shoot_player_cooldown_timer.iter().len() == 0
        {
            let local_bullet_velocity = Vec3 {
                z: -100.0,
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
                Mesh3d(meshes.add(Cuboid {
                    half_size: Vec3::splat(0.05),
                })),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: RED.into(),
                    ..Default::default()
                })),
                LinearVelocity(world_bullet_velocity),
                RigidBody::Kinematic,
                DespawnTimer(Timer::from_seconds(3.0, TimerMode::Once)),
                EnemyBullet,
            ));
            commands.spawn(EnemyShootPlayerCooldownTimer(Timer::from_seconds(
                1.5,
                TimerMode::Once,
            )));
        }
    }
}

fn handle_enemy_shoot_player_cooldown_timer(
    mut commands: Commands,
    timer_query: Query<(Entity, &mut EnemyShootPlayerCooldownTimer)>,
    time: Res<Time>,
) {
    for (entity, mut timer) in timer_query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
