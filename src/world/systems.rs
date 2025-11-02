use std::f32::consts::PI;

use avian3d::prelude::{
    ColliderConstructor, ColliderConstructorHierarchy, RigidBody,
};
use bevy::{
    camera::visibility::RenderLayers,
    color::palettes::{self},
    prelude::*,
};

use crate::{
    game_flow::states::{AppState, SelectedMapState},
    world::{
        MEDIUM_MAP_PATH, SMALL_MAP_PATH,
        collider_rules::get_collider_rules_for_medium_map,
        components::DebugPoint, messages::SpawnDebugPointMessage,
        resources::WorldSceneHandle,
    },
};

pub fn spawn_map(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    selected_map_state: Res<State<SelectedMapState>>,
) {
    info!("Entered LoadingGameSubState::SpawningMap, spawning map");
    let selected_map_state = selected_map_state.get();

    commands.spawn((
        DespawnOnExit(AppState::InGame),
        DirectionalLight {
            illuminance: 4000.,
            shadows_enabled: true,
            color: palettes::css::ANTIQUE_WHITE.into(),
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 12.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        RenderLayers::from_layers(&[0, 1]),
    ));

    let map_path = match selected_map_state {
        SelectedMapState::TinyTown => SMALL_MAP_PATH,
        SelectedMapState::MediumPlastic => MEDIUM_MAP_PATH,
    };
    let world_scene_handle =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(map_path));

    commands.insert_resource(WorldSceneHandle(world_scene_handle.clone()));

    if *selected_map_state == SelectedMapState::MediumPlastic {
        let collider_rules = get_collider_rules_for_medium_map();
        let mut collider_hierarchy = ColliderConstructorHierarchy::new(
            ColliderConstructor::ConvexHullFromMesh,
        );

        for (name, maybe_constructor) in collider_rules {
            match maybe_constructor {
                Some(constructor) => {
                    collider_hierarchy = collider_hierarchy
                        .with_constructor_for_name(name, constructor);
                }
                None => {
                    collider_hierarchy =
                        collider_hierarchy.without_constructor_for_name(name);
                }
            }
        }

        commands.spawn((
            DespawnOnExit(AppState::InGame),
            SceneRoot(world_scene_handle),
            collider_hierarchy,
            Name::new("World Scene Root"),
            Visibility::Visible,
            RigidBody::Static,
        ));
    } else if *selected_map_state == SelectedMapState::TinyTown {
        // Tiny town map has colliders and rigidbody set up in blender as explicit components via skein
        commands.spawn((
            SceneRoot(world_scene_handle),
            Name::new("World Scene Root"),
            Visibility::Visible,
            DespawnOnExit(AppState::InGame),
        ));
    }
}

pub fn handle_spawn_debug_points_message(
    mut commands: Commands,
    mut message_reader: MessageReader<SpawnDebugPointMessage>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for message in message_reader.read() {
        commands.spawn((
            Transform::from_translation(message.point),
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: message.color,
                ..Default::default()
            })),
            DebugPoint,
        ));
    }
}
