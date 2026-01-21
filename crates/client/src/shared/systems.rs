use bevy::{
    camera::visibility::{NoFrustumCulling, RenderLayers},
    mesh::skinning::SkinnedMesh,
    prelude::*,
    scene::SceneInstanceReady,
};

use crate::shared::components::HideOnPause;

/// Currently [`RenderLayers`] are not applied to children of a scene.
/// This [`SceneInstanceReady`] observer applies the [`RenderLayers`]
/// of a [`SceneRoot`] to all children with a [`Transform`] and without a [`RenderLayers`].
///
/// See [#12461](https://github.com/bevyengine/bevy/issues/12461) for current status.
pub fn apply_render_layers_to_children(
    on_scene_instance_ready: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    transforms: Query<&Transform, Without<RenderLayers>>,
    query: Query<(Entity, &RenderLayers)>,
) {
    let Ok((parent, render_layers)) =
        query.get(on_scene_instance_ready.event().entity)
    else {
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

pub fn hide_on_pause(query: Query<&mut Visibility, With<HideOnPause>>) {
    for mut visibility in query {
        *visibility = Visibility::Hidden;
    }
}

pub fn show_on_resume(query: Query<&mut Visibility, With<HideOnPause>>) {
    for mut visibility in query {
        *visibility = Visibility::Visible;
    }
}
