use std::time::Duration;

use bevy::prelude::*;

use crate::{
    enemy::{
        Enemy, EnemyState,
        animate::{
            ENEMY_MODEL_NAME, ENEMY_MODEL_PATH, EnemyAnimationType,
            TOTAL_ENEMY_MODEL_ANIMATIONS,
            get_animation_index_for_enemy_animation_type,
            messages::PlayEnemyAnimationMessage, resources::EnemyAnimations,
        },
    },
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
                    [get_animation_index_for_enemy_animation_type(
                        &EnemyAnimationType::IdleGun,
                    )],
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

pub fn play_enemy_animation(
    animations: Res<EnemyAnimations>,
    mut message_reader: MessageReader<PlayEnemyAnimationMessage>,
    enemy_query: Query<(&Enemy, &AnimationPlayerEntityPointer)>,
    mut animation_players_and_transitions: Query<(
        Entity,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
) {
    for event in message_reader.read() {
        let Ok((enemy, animation_player_entity_pointer)) =
            enemy_query.get(event.enemy)
        else {
            warn!(
                "Tried to play enemy hit animation, but could not find an \
                 Enemy with the entity from the event {} that contains an \
                 AnimationPlayerEntityPointer!",
                event.enemy
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

        let animation_index =
            get_animation_index_for_enemy_animation_type(&event.animaton_type);

        if event.repeat {
            animation_transitions
                .play(
                    &mut animation_player,
                    animations.animation_node_indices[animation_index],
                    Duration::ZERO,
                )
                .repeat();
        } else {
            animation_transitions.play(
                &mut animation_player,
                animations.animation_node_indices[animation_index],
                Duration::ZERO,
            );
        }
    }
}
