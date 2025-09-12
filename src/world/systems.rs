use avian3d::prelude::CollisionStarted;
use bevy::prelude::*;

use crate::{
    common::components::DespawnTimer,
    enemy::EnemyBullet,
    player::shooting::components::PlayerBullet,
    world::components::{Ground, Wall},
};

pub fn setup_world(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("main.gltf")),
    ));
}

pub fn detect_bullet_collision_with_wall_and_grounds(
    mut commands: Commands,
    mut collision_event_reader: EventReader<CollisionStarted>,
    bullet_query: Query<Entity, Or<(With<PlayerBullet>, With<EnemyBullet>)>>,
    wall_and_ground_query: Query<Entity, Or<(With<Wall>, With<Ground>)>>,
) {
    for CollisionStarted(first_entity, second_entity) in
        collision_event_reader.read()
    {
        let is_bullet = bullet_query
            .iter()
            .any(|e| e == *first_entity || e == *second_entity);
        if !is_bullet {
            continue;
        }

        let Some(collided_wall_or_ground_entity) = wall_and_ground_query
            .iter()
            .find(|e| e == first_entity || e == second_entity)
        else {
            continue;
        };

        if collided_wall_or_ground_entity == *first_entity {
            commands.entity(*second_entity).despawn();
        } else {
            commands.entity(*first_entity).despawn();
        }

        info!("Bullet collided with a wall or ground!");
    }
}
