use avian3d::prelude::*;
use bevy::prelude::*;
use shared::{
    components::{DespawnTimer, Health},
    enemy::components::{Enemy, EnemyLastStateUpdate, EnemyState},
    game_score::GameScore,
    player::Player,
    shooting::MAX_SHOOTING_DISTANCE,
    utils::random::get_random_number_from_range,
};

use crate::enemy::shooting::{
    components::EnemyShootCooldownTimer, messages::EnemyKilledMessage,
};

pub fn enemy_shoot_player(
    mut commands: Commands,
    enemy_query: Query<(
        Entity,
        &EnemyState,
        &Transform,
        Option<&EnemyShootCooldownTimer>,
    )>,
    player_transforms: Query<&Transform, With<Player>>,
    spatial_query: SpatialQuery,
    mut health_query: Query<&mut Health>,
) {
    for (
        enemy_entity,
        enemy_state,
        enemy_transform,
        enemy_shoot_cooldown_timer,
    ) in enemy_query
    {
        if *enemy_state != EnemyState::AttackPlayer {
            continue;
        }

        if enemy_shoot_cooldown_timer.is_some() {
            continue;
        }

        let Some(mut closest_player_transform) =
            player_transforms.iter().next()
        else {
            return;
        };

        // for a position X, and an array of positions Y, find the closest position Y to x.
        // TODO: This should be an util
        for player_transform in player_transforms {
            let old_distance = closest_player_transform
                .translation
                .distance(enemy_transform.translation);
            let new_distance = player_transform
                .translation
                .distance(enemy_transform.translation);
            if new_distance < old_distance {
                closest_player_transform = player_transform;
            }
        }

        let random_cooldown = get_random_number_from_range(0.5..1.5);

        commands
            .entity(enemy_entity)
            .insert(EnemyShootCooldownTimer(Timer::from_seconds(
                random_cooldown,
                TimerMode::Repeating,
            )));

        // do raycast from enemy to player direction
        let origin = enemy_transform.translation;

        let random_x_offset = get_random_number_from_range(-0.5..0.5);

        let player_location_random_x_offset =
            closest_player_transform.translation.with_x(
                closest_player_transform.translation.x + random_x_offset as f32,
            );

        let Ok(direction) = Dir3::new(
            player_location_random_x_offset - enemy_transform.translation,
        ) else {
            continue;
        };

        let Some(first_hit) = spatial_query.cast_ray(
            origin,
            direction,
            MAX_SHOOTING_DISTANCE,
            false,
            &SpatialQueryFilter::default()
                .with_excluded_entities([enemy_entity]),
        ) else {
            return;
        };

        if let Ok(mut health) = health_query.get_mut(first_hit.entity) {
            health.0 -= 8.0;

            // FIXME: increase player.death and enemy.kill
            if health.0 <= 0.0 {
                let entity_killed = first_hit.entity;
            }
        }
    }
}

pub fn tick_enemy_shoot_player_cooldown_timer(
    mut commands: Commands,
    timer_query: Query<(Entity, &mut EnemyShootCooldownTimer)>,
    time: Res<Time>,
) {
    for (entity, mut timer) in timer_query {
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            commands.entity(entity).remove::<EnemyShootCooldownTimer>();
        }
    }
}
pub fn detect_killed_enemies(
    changed_enemy_health_query: Query<
        (Entity, &Health),
        (With<Enemy>, Changed<Health>),
    >,
    mut enemy_killed_message_writer: MessageWriter<EnemyKilledMessage>,
) {
    for (enemy_entity, changed_enemy_health) in changed_enemy_health_query {
        if changed_enemy_health.0 <= 0.0 {
            enemy_killed_message_writer.write(EnemyKilledMessage(enemy_entity));
        }
    }
}

pub fn handle_enemy_killed_message(
    mut commands: Commands,
    mut message_reader: MessageReader<EnemyKilledMessage>,
    mut enemy_query: Query<(
        Entity,
        &mut EnemyState,
        &mut EnemyLastStateUpdate,
    )>,
    mut game_score: Single<&mut GameScore>,
) {
    for message in message_reader.read() {
        let Some((enemy_entity, mut enemy_state, mut enemy_last_state_update)) =
            enemy_query
                .iter_mut()
                .find(|(entity, _, _)| *entity == message.0)
        else {
            warn!(
                "An EnemyKilledMessage was read, but the containing enemy \
                 entity does not seem to exist: {}",
                message.0
            );
            continue;
        };

        enemy_state
            .update_state(EnemyState::Dead, &mut enemy_last_state_update);
        commands
            .entity(enemy_entity)
            .remove::<RigidBody>()
            .remove::<Collider>()
            .insert(DespawnTimer(Timer::from_seconds(3.0, TimerMode::Once)));

        game_score.enemies.remove(&enemy_entity);
    }
}
