use std::ops::Neg;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    common::{BULLET_VELOCITY, components::DespawnTimer},
    enemy::{
        Enemy,
        ai::EnemyState,
        shooting::{
            components::{EnemyBullet, EnemyShootPlayerCooldownTimer},
            events::EnemyKilledEvent,
        },
        spawn::{EnemySpawnMethod, SpawnEnemiesEvent},
    },
    game_flow::{
        game_mode::{GameMode, GameStateWave, get_enemy_count_per_wave},
        score::GameScore,
    },
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
    mut enemy_killed_event_writer: EventWriter<EnemyKilledEvent>,
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
            enemy_killed_event_writer.write(EnemyKilledEvent(enemy_entity));
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

// TODO: Does this really belong into the shooting module? its about spawning new enemies and game
// flow/ game mode
pub fn handle_enemy_killed_event(
    mut commands: Commands,
    mut event_reader: EventReader<EnemyKilledEvent>,
    current_game_mode: Res<State<GameMode>>,
    game_state_wave: Res<State<GameStateWave>>,
    mut next_game_state_wave: ResMut<NextState<GameStateWave>>,
    mut enemy_query: Query<(Entity, &mut Enemy)>,
    mut game_score: ResMut<GameScore>,
    mut spawn_enemies_event_writer: EventWriter<SpawnEnemiesEvent>,
) {
    for event in event_reader.read() {
        let Some((enemy_entity, mut enemy)) = enemy_query
            .iter_mut()
            .find(|(entity, _)| *entity == event.0)
        else {
            warn!(
                "An EnemyKilledEvent was fired, but the containing enemy entity does not seem to exist: {}",
                event.0
            );
            continue;
        };

        enemy.state = EnemyState::Dead;
        commands
            .entity(enemy_entity)
            .remove::<RigidBody>()
            .remove::<Collider>()
            .insert(DespawnTimer(Timer::from_seconds(3.0, TimerMode::Once)));

        game_score.player += 1;

        match *current_game_mode.get() {
            GameMode::Waves => {
                let new_enemies_left_count =
                    game_state_wave.enemies_left_from_current_wave - 1;
                next_game_state_wave.set(GameStateWave {
                    current_wave: game_state_wave.current_wave,
                    enemies_left_from_current_wave: new_enemies_left_count,
                });
                if new_enemies_left_count == 0 {
                    info!("no enemies left, spawning new wave!");
                    let new_wave = game_state_wave.current_wave + 1;
                    let enemy_count = get_enemy_count_per_wave(new_wave);
                    next_game_state_wave.set(GameStateWave {
                        current_wave: new_wave,
                        enemies_left_from_current_wave: enemy_count,
                    });
                    spawn_enemies_event_writer.write(SpawnEnemiesEvent {
                        enemy_count,
                        spawn_method: EnemySpawnMethod::RandomSelection,
                    });
                }
            }
            GameMode::None => {}
        }
    }
}
