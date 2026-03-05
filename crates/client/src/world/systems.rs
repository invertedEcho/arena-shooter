use bevy::prelude::*;
use shared::world_object::WorldObjectCollectibleServerSide;

use crate::world::components::FloatDirection;

const MEDKIT_MODEL_PATH: &str = "models/world_objects/medkit.gltf";
const AMMUNITION_MODEL_PATH: &str = "models/world_objects/metal_ammo_box.glb";

pub fn spawn_visuals_for_world_objects(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    added_world_objects: Query<
        (Entity, &WorldObjectCollectibleServerSide),
        Added<WorldObjectCollectibleServerSide>,
    >,
) {
    for (entity, world_object_collectible) in added_world_objects {
        debug!(
            "Spawning world_object_collectible for added \
             WorldObjectCollectibleServerSide {}",
            entity
        );
        let model = match world_object_collectible.kind {
            shared::world_object::WorldObjectCollectibleKind::Medkit => {
                asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset(MEDKIT_MODEL_PATH),
                )
            }
            shared::world_object::WorldObjectCollectibleKind::Ammunition => {
                asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset(AMMUNITION_MODEL_PATH),
                )
            }
        };

        // float direction gets only inserted on the client, we dont want this to be replicated
        // over the network. its only for visuals, so no need for replication.
        commands
            .entity(entity)
            .insert((Visibility::Visible, FloatDirection::Down))
            .with_child((SceneRoot(model), Name::new("World Object Model")));
    }
}

pub fn rotate_and_float_world_objects(
    world_objects_query: Query<
        (&mut FloatDirection, &mut Transform),
        With<WorldObjectCollectibleServerSide>,
    >,
    time: Res<Time>,
) {
    const ORIGIN_Y: f32 = 0.0;
    for (mut float_direction, mut transform) in world_objects_query {
        transform.rotate_y(1. * time.delta_secs());

        let current_y = transform.translation.y;

        match *float_direction {
            FloatDirection::Down => {
                transform.translation.y -= 0.2 * time.delta_secs();
                if ORIGIN_Y - current_y > 0.1 {
                    *float_direction = FloatDirection::Up;
                }
            }
            FloatDirection::Up => {
                transform.translation.y += 0.2 * time.delta_secs();
                if current_y - ORIGIN_Y > 0.1 {
                    *float_direction = FloatDirection::Down;
                }
            }
        }
    }
}

pub fn update_world_object_visibility(
    changed_world_objects: Query<
        (&WorldObjectCollectibleServerSide, &mut Visibility),
        Changed<WorldObjectCollectibleServerSide>,
    >,
) {
    for (changed_world_object, mut visibility) in changed_world_objects {
        if changed_world_object.active {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
