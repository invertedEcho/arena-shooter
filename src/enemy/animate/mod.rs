use std::time::Duration;

use bevy::prelude::*;

use crate::enemy::SWAT_MODEL_PATH;

const TOTAL_ENEMY_MODEL_ANIMATIONS: usize = 24;

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph_handle: Handle<AnimationGraph>,
}

pub struct AnimateEnemyPlugin;

impl Plugin for AnimateEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, animate_enemy);
    }
}

fn setup(
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

    // Keep our animation graph in a Resource so that it can be inserted onto
    // the correct entity once the scene actually loads.
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations {
        animations: node_indices,
        graph_handle,
    });
}

fn animate_enemy(
    mut commands: Commands,
    animations: Res<Animations>,
    players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in players {
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(&mut player, animations.animations[23], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph_handle.clone()))
            .insert(transitions);
    }
}
