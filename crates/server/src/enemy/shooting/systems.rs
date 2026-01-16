use avian3d::prelude::*;
use bevy::prelude::*;
use shared::{components::Health, player::Player};

use crate::enemy::{
    Enemy, EnemyState,
    animate::{EnemyAnimationType, messages::PlayEnemyAnimationMessage},
    shooting::{
        components::EnemyShootCooldownTimer, messages::EnemyKilledMessage,
    },
};

pub fn handle_player_bullet_hit_enemy_message(
    mut enemy_query: Query<(Entity, &mut Health), With<Enemy>>,
    mut player_bullet_hit_enemy_message_reader: MessageReader<
        PlayerBulletHitEnemyMessage,
    >,
    mut enemy_killed_event_writer: MessageWriter<EnemyKilledMessage>,
    mut play_enemy_animation_message_writer: MessageWriter<
        PlayEnemyAnimationMessage,
    >,
) {
    for message in player_bullet_hit_enemy_message_reader.read() {
        let Ok((enemy_entity, mut enemy_health)) =
            enemy_query.get_mut(message.enemy_hit)
        else {
            warn!(
                "Player bullet hit enemy {}, but the enemy entity could not \
                 be found",
                message.enemy_hit
            );
            continue;
        };
        enemy_health.0 -= message.damage;

        if enemy_health.0 > 0.0 {
            play_enemy_animation_message_writer.write(
                PlayEnemyAnimationMessage {
                    enemy: enemy_entity,
                    animaton_type: EnemyAnimationType::HitReceive,
                    repeat: false,
                },
            );
        } else {
            enemy_killed_event_writer.write(EnemyKilledMessage(enemy_entity));
            play_enemy_animation_message_writer.write(
                PlayEnemyAnimationMessage {
                    enemy: enemy_entity,
                    animaton_type: EnemyAnimationType::Death,
                    repeat: false,
                },
            );
        }
    }
}

pub fn enemy_shoot_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    enemy_query: Query<(
        Entity,
        &EnemyState,
        &Transform,
        Option<&EnemyShootCooldownTimer>,
    )>,
    spatial_query: SpatialQuery,
    player_query: Single<(Entity, &Transform, &mut Health), With<Player>>,
    mut debug_gizmos: Option<ResMut<DebugGizmos>>,
) {
    let (player_entity, player_transform, mut player_health) =
        player_query.into_inner();

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

        let player_location_random_x_offset = player_transform
            .translation
            .with_x(player_transform.translation.x + random_x_offset as f32);

        let Ok(direction) = Dir3::new(
            player_location_random_x_offset - enemy_transform.translation,
        ) else {
            continue;
        };

        // debug gizmos may not exist when running release build
        if let Some(ref mut debug_gizmos) = debug_gizmos {
            debug_gizmos.0.push(DebugGizmoLine {
                start: origin,
                end: player_location_random_x_offset,
                despawn_timer: Timer::from_seconds(0.5, TimerMode::Once),
            });
        }

        let Some(first_hit) = spatial_query.cast_ray(
            origin,
            direction,
            500.0,
            false,
            &SpatialQueryFilter::default()
                .with_excluded_entities([enemy_entity]),
        ) else {
            continue;
        };

        if first_hit.entity == player_entity {
            commands.spawn((
                ImageNode {
                    image: asset_server
                        .load("hud/blood_screen_effects/Effect_5.png"),
                    color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                    ..default()
                },
                BloodScreenEffect::default(),
                DespawnOnExit(InGameState::Playing),
            ));

            // TODO: doesnt really make sense to have random number of damage but we do that for now for more "realism"
            let random_damage = get_random_number_from_range(10..15);

            player_health.0 -= random_damage as f32;
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

pub fn handle_enemy_killed_message(
    mut commands: Commands,
    mut message_reader: MessageReader<EnemyKilledMessage>,
    mut enemy_query: Query<(Entity, &mut EnemyState)>,
) {
    for message in message_reader.read() {
        let Some((enemy_entity, mut enemy_state)) = enemy_query
            .iter_mut()
            .find(|(entity, _)| *entity == message.0)
        else {
            warn!(
                "An EnemyKilledMessage was read, but the containing enemy \
                 entity does not seem to exist: {}",
                message.0
            );
            continue;
        };

        enemy_state.update_state(EnemyState::Dead);
        commands
            .entity(enemy_entity)
            .insert(DespawnTimer(Timer::from_seconds(3.0, TimerMode::Once)));
    }
}
