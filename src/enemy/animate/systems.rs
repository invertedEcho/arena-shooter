use std::time::Duration;

use bevy::prelude::*;

use crate::{
    enemy::{
        Enemy, EnemyState,
        animate::{
            ENEMY_DEATH_ANIMATION, ENEMY_HIT_RECEIVE_ANIMATION,
            ENEMY_IDLE_GUN_ANIMATION, ENEMY_IDLE_GUN_POINTING_ANIMATION,
            ENEMY_MODEL_NAME, ENEMY_MODEL_PATH, ENEMY_RUN_ANIMATION,
            TOTAL_ENEMY_MODEL_ANIMATIONS, components::PlayHitAnimationTimer,
            resources::EnemyAnimations,
        },
    },
    player::shooting::messages::PlayerBulletHitEnemyMessage,
    shared::components::AnimationPlayerEntityPointer,
};

pub fn load_enemy_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut animation_clips: Vec<Handle<AnimationClip>> = Vec::new();

    for i in 0..TOTAL_ENEMY_MODEL_ANIMATIONS {
        let res: Handle<AnimationClip> = asset_server
            .load(GltfAssetLabel::Animation(i).from_asset(ENEMY_MODEL_PATH));
        animation_clips.push(res);
    }

    let (graph, node_indices) = AnimationGraph::from_clips(animation_clips);

    let graph_handle = graphs.add(graph);
    commands.insert_resource(EnemyAnimations {
        animation_node_indices: node_indices,
        current_graph_handle: graph_handle,
    });
}

pub fn setup_enemy_animation(
    mut commands: Commands,
    enemy_animations: Res<EnemyAnimations>,
    animation_players: Query<
        (Entity, &mut AnimationPlayer, &Name),
        Added<AnimationPlayer>,
    >,
) {
    for (entity, mut player, name) in animation_players {
        if name.as_str() != ENEMY_MODEL_NAME {
            continue;
        }
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(
                &mut player,
                enemy_animations.animation_node_indices
                    [ENEMY_IDLE_GUN_ANIMATION],
                Duration::ZERO,
            )
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(
                enemy_animations.current_graph_handle.clone(),
            ))
            .insert(transitions);
    }
}

pub fn link_enemy_animation(
    mut commands: Commands,
    animation_player_entities: Query<(Entity, &Name), Added<AnimationPlayer>>,
    enemies: Query<Entity, With<Enemy>>,
    childof: Query<&ChildOf>,
) {
    for (animation_player_entity, name) in &animation_player_entities {
        if name.as_str() != ENEMY_MODEL_NAME {
            continue;
        }
        for ancestor in childof.iter_ancestors(animation_player_entity) {
            if enemies.get(ancestor).is_ok() {
                // ancestor == enemy
                commands
                    .entity(ancestor)
                    .insert(AnimationPlayerEntityPointer(
                        animation_player_entity,
                    ));
                break;
            }
        }
    }
}

pub fn reflect_enemy_state_to_current_animation(
    animations: Res<EnemyAnimations>,
    changed_enemies: Query<
        (&Enemy, &AnimationPlayerEntityPointer),
        Changed<Enemy>,
    >,
    mut animation_players_and_animation_transitions: Query<(
        Entity,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
) {
    for (enemy, animation_player_entity_pointer) in changed_enemies {
        let Some(res) =
            animation_players_and_animation_transitions.iter_mut().find(
                |(entity, _, _)| *entity == animation_player_entity_pointer.0,
            )
        else {
            warn!(
                "could not find animation player and transitions for changed \
                 enemy!"
            );
            continue;
        };

        let (_, mut animation_player, mut animation_transitions) = res;

        let new_animation_index = match enemy.state {
            EnemyState::AttackPlayer => ENEMY_IDLE_GUN_POINTING_ANIMATION,
            EnemyState::Dead => ENEMY_DEATH_ANIMATION,
            EnemyState::GoToLastKnownLocation | EnemyState::PatrolHotspots => {
                ENEMY_RUN_ANIMATION
            }
            EnemyState::CheckIfPlayerSeeable => ENEMY_IDLE_GUN_ANIMATION,
        };

        if new_animation_index == ENEMY_DEATH_ANIMATION {
            animation_transitions.play(
                &mut animation_player,
                animations.animation_node_indices[new_animation_index],
                Duration::from_millis(250),
            );
        } else {
            animation_transitions
                .play(
                    &mut animation_player,
                    animations.animation_node_indices[new_animation_index],
                    Duration::from_millis(250),
                )
                .repeat();
        }
    }
}

pub fn play_enemy_hit_animation(
    mut commands: Commands,
    animations: Res<EnemyAnimations>,
    mut message_reader: MessageReader<PlayerBulletHitEnemyMessage>,
    enemy_query: Query<(Entity, &Enemy, &AnimationPlayerEntityPointer)>,
    mut animation_players_and_transitions: Query<(
        Entity,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
) {
    for event in message_reader.read() {
        let Some((enemy_entity, enemy, animation_player_entity_pointer)) =
            enemy_query.iter().find(|(e, _, _)| *e == event.enemy_hit)
        else {
            warn!(
                "Tried to play enemy hit animation, but could not find an \
                 Enemy with the entity from the event {} that contains an \
                 AnimationPlayerEntityPointer!",
                event.enemy_hit
            );
            continue;
        };
        if enemy.state == EnemyState::Dead {
            continue;
        }

        let Some((_, mut animation_player, mut animation_transitions)) =
            animation_players_and_transitions
                .iter_mut()
                .find(|(e, _, _)| *e == animation_player_entity_pointer.0)
        else {
            warn!(
                "Could not find animation player and transitions for enemy \
                 entity from PlayerBulletHitEnemyMessage"
            );
            continue;
        };

        animation_transitions.play(
            &mut animation_player,
            animations.animation_node_indices[ENEMY_HIT_RECEIVE_ANIMATION],
            Duration::ZERO,
        );

        commands.entity(enemy_entity).insert(PlayHitAnimationTimer(
            Timer::from_seconds(0.5, TimerMode::Once),
        ));
    }
}

// maybe its possible to queue animations? so we dont have to do this manually
pub fn handle_play_hit_animation_timer(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &mut Enemy, &mut PlayHitAnimationTimer)>,
) {
    for (enemy_entity, mut enemy, mut play_hit_animation) in query {
        if enemy.state == EnemyState::Dead {
            continue;
        }
        play_hit_animation.0.tick(time.delta());

        if play_hit_animation.0.just_finished() {
            // TODO: hm we need to set the correct animation to play now depending on the enemystate, but
            // lets not do this here but have a MessageWriter
            // enemy.state = EnemyState::Idle;
            commands
                .entity(enemy_entity)
                .remove::<PlayHitAnimationTimer>();
        }
    }
}
