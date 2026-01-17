use bevy::prelude::*;
use lightyear::prelude::{MessageSender, ReliableSettings};
use shared::{
    components::DespawnTimer, enemy::components::EnemyState, player::Player,
    protocol::ShootRequest, utils::random::get_random_number_from_range,
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
    mut message_sender_shoot_request: Single<&mut MessageSender<ShootRequest>>,
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
            player_transforms.iter().nth(0)
        else {
            return;
        };

        // for a position X, and an array of positions Y, find the closest position Y to x.

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

        message_sender_shoot_request
            .send::<ReliableSettings>(ShootRequest { origin, direction });

        // FIXME: move to client, send a message from server to client to indicate player was hit
        // if first_hit.entity == player_entity {
        //     commands.spawn((
        //         ImageNode {
        //             image: asset_server
        //                 .load("hud/blood_screen_effects/Effect_5.png"),
        //             color: Color::srgba(1.0, 1.0, 1.0, 1.0),
        //             ..default()
        //         },
        //         BloodScreenEffect::default(),
        //         DespawnOnExit(InGameState::Playing),
        //     ));
        //
        //     // TODO: doesnt really make sense to have random number of damage but we do that for now for more "realism"
        //     let random_damage = get_random_number_from_range(10..15);
        //
        //     player_health.0 -= random_damage as f32;
        // }
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
