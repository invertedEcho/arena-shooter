use crate::enemy::shooting::EnemyBullet;
use std::f32::consts::PI;

use avian3d::prelude::CollisionStarted;
use bevy::prelude::*;

use crate::{
    player::shooting::components::PlayerBullet,
    world::components::{Ground, Wall},
};

pub fn setup_world(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
    ));

    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("main.gltf")),
    ));
}

pub fn detect_bullet_collision_with_wall_and_grounds(
    mut commands: Commands,
    mut collision_event_reader: EventReader<CollisionStarted>,
    bullet_query: Query<Entity, Or<(With<PlayerBullet>, With<EnemyBullet>)>>,
    wall_and_ground_query: Query<
        (Entity, &Transform),
        Or<(With<Wall>, With<Ground>)>,
    >,
) {
    for CollisionStarted(first_entity, second_entity) in
        collision_event_reader.read()
    {
        let Some(bullet_entity) = bullet_query
            .iter()
            .find(|e| e == first_entity || e == second_entity)
        else {
            continue;
        };

        let collided_entities_is_wall_or_ground =
            wall_and_ground_query.iter().any(|(entity, _)| {
                entity == *first_entity || entity == *second_entity
            });
        if !collided_entities_is_wall_or_ground {
            continue;
        }

        commands.entity(bullet_entity).despawn();
    }
}
