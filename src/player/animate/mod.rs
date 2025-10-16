use std::time::Duration;

use bevy::{animation::RepeatAnimation, prelude::*};

use crate::{
    common::{MovementState, components::AnimationPlayerEntityPointer},
    player::{Player, movement::PlayerMovementState},
};

const _PLAYER_ARM_WEAPON_DRAW_ANIMATION_INDEX: usize = 0;
const PLAYER_ARM_WEAPON_SHOOT_ANIMATION_INDEX: usize = 1;
const PLAYER_ARM_WEAPON_FULL_RELOAD_ANIMATION_INDEX: usize = 2;
const PLAYER_ARM_WEAPON_IDLE_ANIMATION_INDEX: usize = 3;
const _PLAYER_ARM_WEAPON_INSPECT_ANIMATION_INDEX: usize = 4;
const PLAYER_ARM_WEAPON_PARTIAL_RELOAD_ANIMATION_INDEX: usize = 5;
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
                    link_player_animation,
                    handle_play_arm_with_weapon_animation_event,
                    handle_player_arm_weapon_animation_block_timer,
                ),
            );
    }
}

#[derive(Resource)]
struct PlayerArmWithWeaponAnimations {
    pub animation_node_indices: Vec<AnimationNodeIndex>,
    pub current_graph_handle: Handle<AnimationGraph>,
}

#[derive(Resource)]
pub struct AnimationBlockTimer(pub Timer);

#[derive(Event)]
pub struct PlayArmWithWeaponAnimationEvent {
    pub animation_type: ArmWithWeaponAnimation,
    pub repeat: bool,
    /// If true, this animation will block all other animation requests until the given animation
    /// is done playing. When the animation is done playing, the animation will be played for the
    /// current PlayerMovementState.
    pub block_until_done: bool,
}

#[derive(PartialEq, Debug)]
pub enum ArmWithWeaponAnimation {
    Idle,
    Walk,
    Run,
    Shoot,
    PartialReload,
    FullReload,
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
        // component. we do this because the animationplayer for Enemy has no Name. if we add another
        // animated model which has a name, things will conflict. so we need a better way to get correct
        // AnimationPlayer for the entity we actually want
        (Added<AnimationPlayer>, With<Name>),
    >,
    arm_animations: Res<PlayerArmWithWeaponAnimations>,
) {
    for (entity, mut player) in animation_players {
        debug!("Setting up player arm animation, playing idle on repeat");
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

fn handle_play_arm_with_weapon_animation_event(
    mut commands: Commands,
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
    animation_block_timer: Option<Res<AnimationBlockTimer>>,
) {
    for event in event_reader.read() {
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

        if animation_block_timer.is_some() {
            info!(
                "Got animation request, but the AnimationBlockTimer resource \
                 currently exists, not playing animation {:?}",
                event.animation_type
            );
            return;
        }

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
            ArmWithWeaponAnimation::PartialReload => {
                PLAYER_ARM_WEAPON_PARTIAL_RELOAD_ANIMATION_INDEX
            }
            ArmWithWeaponAnimation::FullReload => {
                PLAYER_ARM_WEAPON_FULL_RELOAD_ANIMATION_INDEX
            }
        };

        info!(
            "Received PlayArmWithWeaponAnimationEvent, playing animation {:?}",
            animation_type
        );

        let repeat_animation_mode = if repeat {
            RepeatAnimation::Forever
        } else {
            RepeatAnimation::Never
        };

        if event.block_until_done {
            let animation_duration =
                get_animation_duration_for_animation_type(animation_type);
            if *animation_type == ArmWithWeaponAnimation::Shoot {
                info!("ANIMATION BLOCK TIMER DURATION: {}", animation_duration);
            }
            commands.insert_resource(AnimationBlockTimer(Timer::from_seconds(
                animation_duration,
                TimerMode::Once,
            )));
        }

        animation_transitions
            .play(
                &mut animation_player,
                animations.animation_node_indices[animation_index],
                Duration::ZERO,
            )
            .set_repeat(repeat_animation_mode);
    }
}

fn handle_player_arm_weapon_animation_block_timer(
    animation_block_timer: Option<ResMut<AnimationBlockTimer>>,
    mut commands: Commands,
    mut event_writer: EventWriter<PlayArmWithWeaponAnimationEvent>,
    time: Res<Time>,
    player_movement_state: Single<&PlayerMovementState>,
) {
    let Some(mut animation_block_timer) = animation_block_timer else {
        return;
    };
    animation_block_timer.0.tick(time.delta());
    if animation_block_timer.0.just_finished() {
        let animation_type = match player_movement_state.0 {
            MovementState::Idle => ArmWithWeaponAnimation::Idle,
            MovementState::Walking => ArmWithWeaponAnimation::Walk,
            MovementState::Running => ArmWithWeaponAnimation::Run,
        };

        // this is race condition, even though we remove the resource, and then write the event,
        // the system says that the PlayArmWithWeaponAnimationEvent still exists
        commands.remove_resource::<AnimationBlockTimer>();
        event_writer.write(PlayArmWithWeaponAnimationEvent {
            animation_type,
            repeat: true,
            block_until_done: false,
        });
    }
}

fn get_animation_duration_for_animation_type(
    animation_type: &ArmWithWeaponAnimation,
) -> f32 {
    match animation_type {
        ArmWithWeaponAnimation::Idle => 3.31,
        ArmWithWeaponAnimation::Walk => 1.31,
        ArmWithWeaponAnimation::Run => 0.48,
        // its actually 0.48 but we want to be able to shoot faster so we use 0.1
        ArmWithWeaponAnimation::Shoot => 0.1,
        ArmWithWeaponAnimation::PartialReload => 2.81,
        ArmWithWeaponAnimation::FullReload => 3.65,
    }
}
