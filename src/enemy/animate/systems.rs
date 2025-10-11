use std::time::Duration;

use bevy::prelude::*;

use crate::{
    enemy::{
        Enemy, EnemyState,
        animate::{
            ENEMY_DEATH_ANIMATION, ENEMY_HIT_RECEIVE_ANIMATION,
            ENEMY_IDLE_GUN_ANIMATION, ENEMY_IDLE_GUN_POINTING_ANIMATION,
            SWAT_MODEL_PATH, TOTAL_ENEMY_MODEL_ANIMATIONS,
            components::{AnimationPlayerEntityPointer, PlayHitAnimationTimer},
            resources::EnemyAnimations,
        },
    },
    player::shooting::events::PlayerBulletHitEnemyEvent,
};

pub fn load_enemy_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut animation_clips: Vec<Handle<AnimationClip>> = Vec::new();

    for i in 0..TOTAL_ENEMY_MODEL_ANIMATIONS {
        let res: Handle<AnimationClip> = asset_server
            .load(GltfAssetLabel::Animation(i).from_asset(SWAT_MODEL_PATH));
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
        (Added<AnimationPlayer>, Without<Name>),
    >,
) {
    for (entity, mut player, name) in animation_players {
        info!("animation player name: {}", name);
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
    animation_player_entities: Query<Entity, Added<AnimationPlayer>>,
    enemies: Query<Entity, With<Enemy>>,
    childof: Query<&ChildOf>,
) {
    for animation_player_entity in &animation_player_entities {
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
            _ => ENEMY_IDLE_GUN_ANIMATION,
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
    mut event_reader: EventReader<PlayerBulletHitEnemyEvent>,
    animation_player_entity_pointers: Query<
        (Entity, &Enemy, &AnimationPlayerEntityPointer),
        With<Enemy>,
    >,
    mut animation_players_and_transitions: Query<(
        Entity,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
) {
    for event in event_reader.read() {
        let Some((enemy_entity, enemy, animation_player_entity_pointer)) =
            animation_player_entity_pointers
                .iter()
                .find(|(e, _, _)| *e == event.enemy_hit)
        else {
            warn!("lksjdfjkldfs");
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
                 entity from PlayerBulletHitEnemyEvent"
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
pub fn handle_play_hit_animation(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(
        Entity,
        &Enemy,
        &mut PlayHitAnimationTimer,
        &AnimationPlayerEntityPointer,
    )>,
    mut animation_players_and_transitions: Query<(
        Entity,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
    animations: Res<EnemyAnimations>,
) {
    for (
        enemy_entity,
        enemy,
        mut play_hit_animation,
        animation_player_entity_pointer,
    ) in query
    {
        if enemy.state == EnemyState::Dead {
            continue;
        }
        play_hit_animation.0.tick(time.delta());

        if !play_hit_animation.0.just_finished() {
            continue;
        }

        let Some((_, mut animation_player, mut animation_transitions)) =
            animation_players_and_transitions
                .iter_mut()
                .find(|(e, _, _)| *e == animation_player_entity_pointer.0)
        else {
            warn!(
                "Could not find animation player and transitions for enemy \
                 entity"
            );
            continue;
        };
        let new_animation_index = match enemy.state {
            EnemyState::AttackPlayer => ENEMY_IDLE_GUN_POINTING_ANIMATION,
            EnemyState::Idle | EnemyState::ChasingPlayer | EnemyState::Dead => {
                ENEMY_IDLE_GUN_ANIMATION
            }
        };
        animation_transitions.play(
            &mut animation_player,
            animations.animation_node_indices[new_animation_index],
            Duration::from_millis(250),
        );
        commands
            .entity(enemy_entity)
            .remove::<PlayHitAnimationTimer>();
    }
}
