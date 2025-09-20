use std::time::Duration;

use bevy::{animation::AnimationTarget, prelude::*};

use crate::enemy::{Enemy, EnemyState, SWAT_MODEL_PATH};

const TOTAL_ENEMY_MODEL_ANIMATIONS: usize = 24;
const IDLE_ANIMATION: usize = 2;

#[derive(Resource)]
struct EnemyAnimations {
    animation_node_indices: Vec<AnimationNodeIndex>,
    current_graph_handle: Handle<AnimationGraph>,
}

pub struct AnimateEnemyPlugin;

impl Plugin for AnimateEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_enemy_animations).add_systems(
            Update,
            (
                setup_enemy_animation,
                // update_enemy_animation_on_state_changed,
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
        (Entity, &mut AnimationPlayer, &AnimationTarget, &Children),
        Added<AnimationPlayer>,
    >,
) {
    for (entity, mut player, animation_target, children) in animation_players {
        info!("animation_target: {:?}", animation_target.player);
        info!("children: {:?}", children);
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(
                &mut player,
                enemy_animations.animation_node_indices[IDLE_ANIMATION],
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

// fn update_enemy_animation_on_state_changed(
//     mut commands: Commands,
//     animations: Res<EnemyAnimations>,
//     changed_enemies: Query<(&Enemy, &Children, Entity), Changed<Enemy>>,
//     mut animation_players_and_animation_transitions: Query<(
//         &mut AnimationPlayer,
//         &mut AnimationTransitions,
//     )>,
//     res: Query<&AnimationPlayer>,
// ) {
//     for (enemy, children, entity) in changed_enemies {
//         info!("found enemy changed that has children");
//         for child in children {
//             commands.entity(*child).log_components();
//             match animation_players_and_animation_transitions.get_mut(*child) {
//                 Ok(res) => {
//                     info!("found");
//                 }
//                 Err(error) => {
//                     error!("{}", error)
//                 }
//             }
//             if let Ok((mut animation_player, mut transitions)) =
//                 animation_players_and_animation_transitions.get_mut(*child)
//             {
//                 let new_animation_index = match enemy.state {
//                     EnemyState::AttackPlayer => 3,
//                     _ => IDLE_ANIMATION,
//                 };
//                 transitions
//                     .play(
//                         &mut animation_player,
//                         animations.animation_node_indices[new_animation_index],
//                         Duration::from_millis(250),
//                     )
//                     .repeat();
//             } else {
//                 info!("couldnt find");
//             }
//         }
//     }
// }
