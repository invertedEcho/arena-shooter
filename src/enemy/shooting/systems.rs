use std::ops::Neg;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    common::{BULLET_VELOCITY, components::DespawnTimer},
    enemy::{
        Enemy,
        ai::EnemyState,
        shooting::components::{EnemyBullet, EnemyShootPlayerCooldownTimer},
        spawn::SpawnEnemiesAtSpawnLocationsEvent,
    },
    game_flow::score::GameScore,
    player::shooting::{
        components::PlayerBullet, events::PlayerBulletHitEnemyEvent,
    },
};

pub fn detect_player_bullet_collision_with_enemy(
    mut commands: Commands,
    player_bullet_query: Query<(Entity, &PlayerBullet)>,
    mut enemy_query: Query<(Entity, &mut Enemy)>,
    mut collision_event_reader: EventReader<CollisionStarted>,
    mut player_bullet_hit_enemy_event_writer: EventWriter<
        PlayerBulletHitEnemyEvent,
    >,
    mut game_score: ResMut<GameScore>,
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

        let Some((enemy_entity, mut enemy)) =
            enemy_query.iter_mut().find(|(entity, _)| {
                entity == first_entity || entity == second_entity
            })
        else {
            continue;
        };

        enemy.health -= player_bullet.1.damage;
        if enemy.health <= 0.0 {
            enemy.state = EnemyState::Dead;
            // TODO: perhaps this should be handled elsewhere.
            // move this elsewhere when we do even more stuff here on initial death of enemy
            commands
                .entity(enemy_entity)
                .remove::<RigidBody>()
                .remove::<Collider>();
            game_score.player += 1;
            commands.entity(enemy_entity).insert(DespawnTimer(
                Timer::from_seconds(3.0, TimerMode::Once),
            ));
        }
        commands.entity(player_bullet.0).despawn();

        player_bullet_hit_enemy_event_writer.write(PlayerBulletHitEnemyEvent {
            enemy_hit: enemy_entity,
        });
    }
}

pub fn enemy_shoot_player(
    mut commands: Commands,
    enemy_query: Query<(&Enemy, &Transform, &EnemyShootPlayerCooldownTimer)>,
) {
    for (enemy, enemy_transform, enemy_shoot_player_cooldown_timer) in
        enemy_query
    {
        if enemy.state == EnemyState::Dead {
            continue;
        }

        if enemy.state != EnemyState::AttackPlayer
            || !enemy_shoot_player_cooldown_timer.0.just_finished()
        {
            continue;
        }

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

pub fn tick_enemy_shoot_player_cooldown_timer(
    timer_query: Query<&mut EnemyShootPlayerCooldownTimer>,
    time: Res<Time>,
) {
    for mut timer in timer_query {
        timer.0.tick(time.delta());
    }
}
