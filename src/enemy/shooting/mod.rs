use std::ops::Neg;

use crate::common::components::DespawnTimer;
use crate::enemy::BULLET_VELOCITY;
use crate::enemy::EnemyAiState;
use crate::enemy::animate::play_enemy_hit_animation;
use crate::enemy::spawn::SpawnEnemiesAtSpawnLocationsEvent;
use crate::game_flow::score::GameScore;
use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    enemy::Enemy,
    game_flow::GameState,
    player::shooting::{
        components::PlayerBullet, events::PlayerBulletHitEnemyEvent,
    },
};

pub struct EnemyShootingPlugin;

impl Plugin for EnemyShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                enemy_shoot_player,
                tick_enemy_shoot_player_cooldown_timer,
                // TODO: the detect_player_bullet_collision_with_enemy system may despawn an entity
                // that the play_enemy_hit_animation system will insert into. i think we could just
                // solve this issue with animating the enemy death, and at animation end then
                // despawn
                detect_player_bullet_collision_with_enemy
                    .after(play_enemy_hit_animation),
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct EnemyBullet;

#[derive(Component)]
pub struct EnemyShootPlayerCooldownTimer(pub Timer);

fn detect_player_bullet_collision_with_enemy(
    mut commands: Commands,
    player_bullet_query: Query<(Entity, &PlayerBullet)>,
    mut enemy_query: Query<(Entity, &mut Enemy)>,
    mut collision_event_reader: EventReader<CollisionStarted>,
    mut player_bullet_hit_enemy_event_writer: EventWriter<
        PlayerBulletHitEnemyEvent,
    >,
    mut spawn_enemies_event_writer: EventWriter<
        SpawnEnemiesAtSpawnLocationsEvent,
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

        let count_of_enemies = enemy_query.iter().len();
        let Some(mut enemy) = enemy_query.iter_mut().find(|(entity, _)| {
            entity == first_entity || entity == second_entity
        }) else {
            continue;
        };

        enemy.1.health -= player_bullet.1.damage;
        if enemy.1.health <= 0.0 {
            commands.entity(enemy.0).despawn();
            game_score.player += 1;

            // check if this was the last enemy
            if count_of_enemies == 1 {
                spawn_enemies_event_writer
                    .write(SpawnEnemiesAtSpawnLocationsEvent);
                commands
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::End,
                            align_items: AlignItems::End,
                            ..default()
                        },
                        DespawnTimer(Timer::from_seconds(3.0, TimerMode::Once)),
                    ))
                    .with_child(Text::new(
                        "A new wave of enemies has been spawned!",
                    ));
            }
        }
        commands.entity(player_bullet.0).despawn();

        player_bullet_hit_enemy_event_writer
            .write(PlayerBulletHitEnemyEvent { enemy_hit: enemy.0 });
    }
}

fn enemy_shoot_player(
    mut commands: Commands,
    enemy_query: Query<(&Enemy, &Transform, &EnemyShootPlayerCooldownTimer)>,
) {
    for (enemy, enemy_transform, enemy_shoot_player_cooldown_timer) in
        enemy_query
    {
        if enemy.state == EnemyAiState::AttackPlayer
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
