use avian3d::prelude::CollisionStarted;
use bevy::prelude::*;

use crate::{
    enemy::EnemyBullet,
    particles::SpawnBulletImpactEffectEvent,
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
    wall_and_ground_query: Query<
        (Entity, &Transform),
        Or<(With<Wall>, With<Ground>)>,
    >,
    mut bullet_effect_spawn_event_writer: EventWriter<
        SpawnBulletImpactEffectEvent,
    >,
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

        let Some(collided_wall_or_ground_entity) =
            wall_and_ground_query.iter().find(|(entity, _)| {
                entity == first_entity || entity == second_entity
            })
        else {
            continue;
        };

        if collided_wall_or_ground_entity.0 == *first_entity {
            commands.entity(*second_entity).despawn();
        } else {
            commands.entity(*first_entity).despawn();
        }

        // well this doesnt work. the transform is just the center of the hit collider, but not the
        // actual point. what about using raycasts instead, e.g. when player shoots, send a raycast
        // in the direction he is looking, and then check if collided with a wall or ground, and
        // then we also get accurate location?
        bullet_effect_spawn_event_writer.write(SpawnBulletImpactEffectEvent {
            spawn_location: collided_wall_or_ground_entity.1.translation,
        });
    }
}
