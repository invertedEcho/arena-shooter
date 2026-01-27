use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::{
    camera::visibility::RenderLayers,
    color::palettes::{self},
    prelude::*,
};
use shared::{MEDIUM_PLASTIC_MAP_PATH, SelectedMapState, TINY_TOWN_MAP_PATH};

use super::resources::WorldSceneHandle;
use crate::{
    game_flow::states::AppState,
    world::components::{MapDirectionalLight, MapModel},
};

/// Spawns the corresponding map (determined by looking at SelectedMapState) on the client, when
/// we enter LoadingGameState::SpawningMap
pub fn on_enter_spawn_map(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    selected_map_state: Res<State<SelectedMapState>>,
) {
    let selected_map = selected_map_state.get();
    let map_path = match selected_map {
        SelectedMapState::TinyTown => TINY_TOWN_MAP_PATH,
        SelectedMapState::MediumPlastic => MEDIUM_PLASTIC_MAP_PATH,
    };

    info!(
        "Entered LoadingGameState::SpawningMap, spawning map {:?} with path \
         {:?}",
        selected_map, map_path
    );

    commands.spawn((
        DespawnOnExit(AppState::InGame),
        DirectionalLight {
            illuminance: 4000.,
            shadows_enabled: true,
            color: palettes::css::ANTIQUE_WHITE.into(),
            ..default()
        },
        MapDirectionalLight,
        Transform {
            translation: Vec3::new(0.0, 12.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
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
