use bevy::{
    camera::visibility::{NoFrustumCulling, RenderLayers},
    mesh::skinning::SkinnedMesh,
    prelude::*,
    scene::SceneInstanceReady,
};

use super::components::DespawnTimer;

pub fn handle_despawn_timer(
    despawn_timer_query: Query<(Entity, &mut DespawnTimer)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut timer) in despawn_timer_query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Currently [`RenderLayers`] are not applied to children of a scene.
/// This [`SceneInstanceReady`] observer applies the [`RenderLayers`]
/// of a [`SceneRoot`] to all children with a [`Transform`] and without a [`RenderLayers`].
///
/// See [#12461](https://github.com/bevyengine/bevy/issues/12461) for current status.
pub fn apply_render_layers_to_children(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    transforms: Query<&Transform, Without<RenderLayers>>,
    query: Query<(Entity, &RenderLayers)>,
) {
    let Ok((parent, render_layers)) = query.get(trigger.target()) else {
        return;
    };
    children.iter_descendants(parent).for_each(|entity| {
        if transforms.contains(entity) {
            commands.entity(entity).insert(render_layers.clone());
        }
    });
}

// https://github.com/bevyengine/bevy/issues/4971
pub fn disable_culling_for_skinned_meshes(
    mut commands: Commands,
    skinned: Query<Entity, Added<SkinnedMesh>>,
) {
    for entity in &skinned {
        commands.entity(entity).insert(NoFrustumCulling);
    }
}
