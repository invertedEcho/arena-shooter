use std::{fs::File, io::Read};

use avian3d::math::PI;
use avian3d::prelude::*;
use bevy::{color::palettes, prelude::*};
use shared::MEDIUM_PLASTIC_MAP_PATH;

use crate::utils::get_path_to_collider_json;

pub fn spawn_map(asset_server: Res<AssetServer>, mut commands: Commands) {
    info!("Spawning map on server");

    let map_path = MEDIUM_PLASTIC_MAP_PATH;

    commands.spawn((
        Name::new("Map Light"),
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
    ));

    let world_scene_handle =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(map_path));

    commands.spawn((
        SceneRoot(world_scene_handle),
        Name::new("Medium Plastic Map Scene Root"),
        Visibility::Visible,
        RigidBody::Static,
    ));
}

pub fn spawn_map_colliders(mut commands: Commands) {
    let file_path = get_path_to_collider_json();

    let mut file_buffer = String::from("");
    let mut collider_file =
        File::open(file_path).expect("Can open medium_plastic_colliders.json");

    collider_file.read_to_string(&mut file_buffer).unwrap();

    let colliders: Result<
        Vec<(Collider, GlobalTransform)>,
        serde_json::error::Error,
    > = serde_json::from_str(&file_buffer);

    match colliders {
        Ok(colliders_ok) => {
            info!(
                "Loaded colliders and their transform from json, spawning \
                 them."
            );
            commands.spawn_batch(colliders_ok);
        }
        Err(error) => {
            panic!(
                "Failed to load colliders and their transform from json: {}",
                error
            );
        }
    }
}
