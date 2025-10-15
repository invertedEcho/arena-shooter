use std::time::Duration;

use bevy::{animation::RepeatAnimation, prelude::*};

use crate::{
    common::{MovementState, components::AnimationPlayerEntityPointer},
    player::{Player, movement::PlayerMovementState},
};

const PLAYER_ARM_WEAPON_RELOAD_ANIMATION_INDEX: usize = 0;
const PLAYER_ARM_WEAPON_SHOOT_ANIMATION_INDEX: usize = 1;
const PLAYER_ARM_WEAPON_IDLE_ANIMATION_INDEX: usize = 3;
const PLAYER_ARM_WEAPON_WALK_ANIMATION_INDEX: usize = 6;
const PLAYER_ARM_WEAPON_RUN_ANIMATION_INDEX: usize = 7;

pub struct PlayerAnimatePlugin;

impl Plugin for PlayerAnimatePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayArmWithWeaponAnimationEvent>()
            .add_systems(Startup, load_arms_animations)
            .add_systems(
                Update,
                (
                    setup_arm_animation,
                    // reflect_player_state_to_current_arm_animation,
                    link_player_animation,
                    handle_play_arm_with_weapon_animation_event,
                ),
            );
    }
}

#[derive(Resource)]
struct PlayerArmWithWeaponAnimations {
    pub animation_node_indices: Vec<AnimationNodeIndex>,
    pub current_graph_handle: Handle<AnimationGraph>,
}

#[derive(Event)]
pub struct PlayArmWithWeaponAnimationEvent {
    pub animation_type: ArmWithWeaponAnimation,
    pub repeat: bool,
}

pub enum ArmWithWeaponAnimation {
    Idle,
    Walk,
    Run,
    Shoot,
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
    commands.insert_resource(PlayerArmWithWeaponAnimations {
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
    arm_animations: Res<PlayerArmWithWeaponAnimations>,
) {
    for (entity, mut player) in animation_players {
        info!("Setting up player arm animation, playing idle on repeat");
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(
                &mut player,
                arm_animations.animation_node_indices
                    [PLAYER_ARM_WEAPON_IDLE_ANIMATION_INDEX],
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
                info!(
                    "Inserted animation player entity pointer into Player {}",
                    *player
                );
                commands
                    .entity(ancestor)
                    .insert(AnimationPlayerEntityPointer(
                        animation_player_entity,
                    ));
            }
        }
    }
}

// fn reflect_player_state_to_current_arm_animation(
//     animations: Res<PlayerArmWithWeaponAnimations>,
//     player_query: Query<
//         (&AnimationPlayerEntityPointer, &PlayerMovementState),
//         Changed<PlayerMovementState>,
//     >,
//     mut animation_players_and_animation_transitions: Query<(
//         Entity,
//         &mut AnimationPlayer,
//         &mut AnimationTransitions,
//     )>,
// ) {
//     for (animation_player_entity_pointer, player_movement_state) in player_query
//     {
//         let Ok((_, mut animation_player, mut animation_transitions)) =
//             animation_players_and_animation_transitions
//                 .get_mut(animation_player_entity_pointer.0)
//         else {
//             warn!(
//                 "Failed to find animation player for entity pointer in arm \
//                  animations"
//             );
//             return;
//         };
//         let animation_index = match player_movement_state.0 {
//             MovementState::Idle => PLAYER_ANIMATION_INDEX_IDLE,
//             MovementState::Walking => PLAYER_ANIMATION_INDEX_RUN,
//             MovementState::Running => PLAYER_ANIMATION_INDEX_SPRINT,
//         };
//
//         info!(
//             "Reflecting movement state change to player arm weapon animation, \
//              index: {}",
//             animation_index
//         );
//
//         animation_transitions
//             .play(
//                 &mut animation_player,
//                 animations.animation_node_indices[animation_index],
//                 Duration::ZERO,
//             )
//             .repeat();
//     }
// }

fn handle_play_arm_with_weapon_animation_event(
    mut event_reader: EventReader<PlayArmWithWeaponAnimationEvent>,
    animation_player_entity_pointer: Single<
        &AnimationPlayerEntityPointer,
        With<Player>,
    >,
    mut animation_players_and_animation_transitions: Query<(
        Entity,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
    animations: Res<PlayerArmWithWeaponAnimations>,
) {
    for event in event_reader.read() {
        let animation_type = &event.animation_type;
        let repeat = event.repeat;

        let animation_index = match animation_type {
            ArmWithWeaponAnimation::Shoot => {
                PLAYER_ARM_WEAPON_SHOOT_ANIMATION_INDEX
            }
            ArmWithWeaponAnimation::Idle => {
                PLAYER_ARM_WEAPON_IDLE_ANIMATION_INDEX
            }
            ArmWithWeaponAnimation::Walk => {
                PLAYER_ARM_WEAPON_WALK_ANIMATION_INDEX
            }
            ArmWithWeaponAnimation::Run => {
                PLAYER_ARM_WEAPON_RUN_ANIMATION_INDEX
            }
        };

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
        info!("received event, playing animation {}", animation_index);
        let repeat_animation_mode = if repeat {
            RepeatAnimation::Forever
        } else {
            RepeatAnimation::Never
        };

        animation_transitions
            .play(
                &mut animation_player,
                animations.animation_node_indices[animation_index],
                Duration::ZERO,
            )
            .set_repeat(repeat_animation_mode);
    }
}
