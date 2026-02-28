use avian3d::prelude::*;
use bevy::{camera::visibility::RenderLayers, prelude::*};
use shared::{
    MEDIUM_PLASTIC_MAP_PATH, SelectedMapState, TINY_TOWN_MAP_PATH,
    world_object::WorldObjectCollectibleServerSide,
};

use super::resources::WorldSceneHandle;
use crate::{
    game_flow::states::{AppState, GameModeClient},
    world::components::{FloatDirection, MapDirectionalLight, MapModel},
};

/// Spawns the corresponding map (determined by looking at SelectedMapState) on the client, when
/// we enter LoadingGameState::SpawningMap
pub fn on_enter_spawn_map(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    selected_map_state: Res<State<SelectedMapState>>,
    selected_game_mode: Res<State<GameModeClient>>,
) {
    let selected_map = selected_map_state.get();

    let map_path = if *selected_game_mode == GameModeClient::Multiplayer {
        MEDIUM_PLASTIC_MAP_PATH
    } else {
        match selected_map {
            SelectedMapState::TinyTown => TINY_TOWN_MAP_PATH,
            SelectedMapState::MediumPlastic => MEDIUM_PLASTIC_MAP_PATH,
        }
    };

    info!(
        "Entered LoadingGameState::SpawningMap, spawning map {:?} with path \
         {:?}",
        selected_map, map_path
    );

    commands.spawn((
        DespawnOnExit(AppState::InGame),
        DirectionalLight {
            illuminance: 6000.,
            shadows_enabled: true,
            ..default()
        },
        MapDirectionalLight,
        Transform::default().looking_at(Vec3::new(-1.0, -3.0, -2.0), Vec3::Y),
        // TODO: should be constants
        RenderLayers::from_layers(&[0, 1]),
    ));

    let world_scene_handle =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(map_path));

    commands.insert_resource(WorldSceneHandle(world_scene_handle.clone()));

    commands.spawn((
        DespawnOnExit(AppState::InGame),
        SceneRoot(world_scene_handle),
        Name::new("Scene Root (Map)"),
        Visibility::Visible,
        RigidBody::Static,
        MapModel,
    ));
}

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
        let model = match world_object_collectible.object_type {
            shared::world_object::WorldObjectCollectibleType::Medkit => {
                asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset(MEDKIT_MODEL_PATH),
                )
            }
            shared::world_object::WorldObjectCollectibleType::Ammunition => {
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
