use bevy::prelude::*;

#[derive(Resource)]
pub struct EnemyAnimations {
    pub animation_node_indices: Vec<AnimationNodeIndex>,
    pub current_graph_handle: Handle<AnimationGraph>,
}
