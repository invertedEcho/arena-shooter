use std::time::Duration;

use bevy::prelude::*;

use crate::{
    common::{MovementState, components::AnimationPlayerEntityPointer},
    player::{Player, movement::PlayerMovementState},
};

const PLAYER_ANIMATION_INDEX_IDLE: usize = 3;
const PLAYER_ANIMATION_INDEX_RUN: usize = 6;
const PLAYER_ANIMATION_INDEX_SPRINT: usize = 7;

pub struct PlayerAnimatePlugin;

impl Plugin for PlayerAnimatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_arms_animations).add_systems(
            Update,
            (
                setup_arm_animation,
                reflect_player_state_to_current_arm_animation,
                link_player_animation,
            ),
        );
    }
}

#[derive(Resource)]
struct PlayerArmAnimations {
    pub animation_node_indices: Vec<AnimationNodeIndex>,
    pub current_graph_handle: Handle<AnimationGraph>,
}

fn load_arms_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut animation_clips: Vec<Handle<AnimationClip>> = Vec::new();

    for i in 0..9 {
        let res: Handle<AnimationClip> = asset_server
            .load(GltfAssetLabel::Animation(i).from_asset("test.glb"));
        animation_clips.push(res);
    }

    let (graph, node_indices) = AnimationGraph::from_clips(animation_clips);

    let graph_handle = graphs.add(graph);
    commands.insert_resource(PlayerArmAnimations {
        animation_node_indices: node_indices,
        current_graph_handle: graph_handle,
    });
}

fn setup_arm_animation(
    mut commands: Commands,
    animation_players: Query<
        (Entity, &mut AnimationPlayer),
        // TODO: this is kinda risky, we pretty much depend on this AnimationPlayer entity having the name
        // component. we do this because the animationplayer forr enemy has no Name. if we add another
        // animated model which has a name, things will conflict. so we neeed a better way to get correct
        // AnimationPlayer for the entity we actually want
        (Added<AnimationPlayer>, With<Name>),
    >,
    arm_animations: Res<PlayerArmAnimations>,
) {
    for (entity, mut player) in animation_players {
        info!("Setting up player arm animation");
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(
                &mut player,
                arm_animations.animation_node_indices
                    [PLAYER_ANIMATION_INDEX_IDLE],
                Duration::ZERO,
            )
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(
                arm_animations.current_graph_handle.clone(),
            ))
            .insert(transitions);
    }
}

pub fn link_player_animation(
    mut commands: Commands,
    animation_player_entities: Query<
        Entity,
        (Added<AnimationPlayer>, With<Name>),
    >,
    player: Single<Entity, With<Player>>,
    child_of: Query<&ChildOf>,
) {
    for animation_player_entity in &animation_player_entities {
        for ancestor in child_of.iter_ancestors(animation_player_entity) {
            if *player == ancestor {
                info!("inserted animation player entity pointer into player");
                commands
                    .entity(ancestor)
                    .insert(AnimationPlayerEntityPointer(
                        animation_player_entity,
                    ));
            }
        }
    }
}

fn reflect_player_state_to_current_arm_animation(
    animations: Res<PlayerArmAnimations>,
    player_query: Single<
        (&AnimationPlayerEntityPointer, &PlayerMovementState),
        Changed<PlayerMovementState>,
    >,
    mut animation_players_and_animation_transitions: Query<(
        Entity,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
) {
    let (animation_player_entity_pointer, player_movement_state) =
        *player_query;
    info!("player movement state was changed!");
    let Ok((_, mut animation_player, mut animation_transitions)) =
        animation_players_and_animation_transitions
            .get_mut(animation_player_entity_pointer.0)
    else {
        warn!(
            "Failed to find animation player for entity pointer in arm \
             animations"
        );
        return;
    };
    let animation_index = match player_movement_state.0 {
        MovementState::Idle => PLAYER_ANIMATION_INDEX_IDLE,
        MovementState::Walking => PLAYER_ANIMATION_INDEX_RUN,
        MovementState::Running => PLAYER_ANIMATION_INDEX_SPRINT,
    };
    info!("playing animation index: {}", animation_index);

    animation_transitions
        .play(
            &mut animation_player,
            animations.animation_node_indices[animation_index],
            Duration::ZERO,
        )
        .repeat();
}
