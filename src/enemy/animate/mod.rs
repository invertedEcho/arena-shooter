use std::time::Duration;

use bevy::prelude::*;

use crate::{
    enemy::{
        Enemy, EnemyAiState,
        animate::{
            components::{AnimationPlayerEntityPointer, PlayHitAnimation},
            resources::EnemyAnimations,
        },
    },
    player::shooting::events::PlayerBulletHitEnemyEvent,
};

mod components;
mod resources;

const TOTAL_ENEMY_MODEL_ANIMATIONS: usize = 24;
// https://poly.pizza/m/Btfn3G5Xv4 index is equal to list option select thing on preview
const _ENEMY_DEATH_ANIMATION: usize = 0;
const _ENEMY_GUN_SHOOT_ANIMATION: usize = 1;
const ENEMY_HIT_RECEIVE_ANIMATION: usize = 2;
const _ENEMY_IDLE_ANIMATION: usize = 4;
const ENEMY_IDLE_GUN_ANIMATION: usize = 5;
const ENEMY_IDLE_GUN_POINTING_ANIMATION: usize = 6;

pub const SWAT_MODEL_PATH: &str = "models/animated/SWAT.glb";

pub struct AnimateEnemyPlugin;

impl Plugin for AnimateEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_enemy_animations).add_systems(
            Update,
            (
                setup_enemy_animation,
                update_enemy_animation_on_state_changed,
                link_enemy_animation,
                play_enemy_hit_animation,
                handle_play_hit_animation,
            ),
        );
    }
}

fn load_enemy_animations(
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

fn setup_enemy_animation(
    mut commands: Commands,
    enemy_animations: Res<EnemyAnimations>,
    animation_players: Query<
        (Entity, &mut AnimationPlayer),
        Added<AnimationPlayer>,
    >,
) {
    for (entity, mut player) in animation_players {
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

fn link_enemy_animation(
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

fn update_enemy_animation_on_state_changed(
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
                "could not find animation player and transitions for changed enemy!"
            );
            continue;
        };

        let (_, mut animation_player, mut animation_transitions) = res;

        let new_animation_index = match enemy.state {
            EnemyAiState::AttackPlayer => ENEMY_IDLE_GUN_POINTING_ANIMATION,
            _ => ENEMY_IDLE_GUN_ANIMATION,
        };
        animation_transitions
            .play(
                &mut animation_player,
                animations.animation_node_indices[new_animation_index],
                Duration::from_millis(250),
            )
            .repeat();
    }
}

fn play_enemy_hit_animation(
    mut commands: Commands,
    animations: Res<EnemyAnimations>,
    mut event_reader: EventReader<PlayerBulletHitEnemyEvent>,
    animation_player_entity_pointers: Query<
        (Entity, &AnimationPlayerEntityPointer),
        With<Enemy>,
    >,
    mut animation_players_and_transitions: Query<(
        Entity,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
) {
    for event in event_reader.read() {
        let Some(enemy) = animation_player_entity_pointers
            .iter()
            .find(|(e, _)| *e == event.enemy_hit)
        else {
            warn!("lksjdfjkldfs");
            continue;
        };

        let Some((_, mut animation_player, mut animation_transitions)) =
            animation_players_and_transitions
                .iter_mut()
                .find(|(e, _, _)| *e == enemy.1.0)
        else {
            warn!(
                "Could not find animation player and transitions for enemy entity from PlayerBulletHitEnemyEvent"
            );
            continue;
        };

        animation_transitions.play(
            &mut animation_player,
            animations.animation_node_indices[ENEMY_HIT_RECEIVE_ANIMATION],
            Duration::ZERO,
        );

        // after 0.5 seconds play normal animation again
        commands
            .entity(enemy.0)
            .insert(PlayHitAnimation(Timer::from_seconds(
                0.5,
                TimerMode::Once,
            )));
    }
}

// maybe its possible to queue animations? so we dont have to do this manually
fn handle_play_hit_animation(
    time: Res<Time>,
    query: Query<(
        &Enemy,
        &mut PlayHitAnimation,
        &AnimationPlayerEntityPointer,
    )>,
    mut animation_players_and_transitions: Query<(
        Entity,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
    animations: Res<EnemyAnimations>,
) {
    for (enemy, mut play_hit_animation, animation_player_entity_pointer) in
        query
    {
        play_hit_animation.0.tick(time.delta());

        let Some((_, mut animation_player, mut animation_transitions)) =
            animation_players_and_transitions
                .iter_mut()
                .find(|(e, _, _)| *e == animation_player_entity_pointer.0)
        else {
            warn!(
                "Could not find animation player and transitions for enemy entity"
            );
            continue;
        };

        let new_animation_index = match enemy.state {
            EnemyAiState::AttackPlayer => ENEMY_IDLE_GUN_POINTING_ANIMATION,
            EnemyAiState::Idle | EnemyAiState::ChasingPlayer => {
                ENEMY_IDLE_GUN_ANIMATION
            }
        };

        if play_hit_animation.0.just_finished() {
            animation_transitions.play(
                &mut animation_player,
                animations.animation_node_indices[new_animation_index],
                Duration::from_millis(250),
            );
        }
    }
}
