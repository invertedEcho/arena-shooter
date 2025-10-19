use std::f32::consts::PI;

use bevy::{camera::visibility::RenderLayers, color::palettes, prelude::*};

use crate::{
    enemy::shooting::components::EnemyBullet,
    player::shooting::components::PlayerBullet,
    world::components::{Ground, Wall},
};

pub fn setup_world(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 3000.,
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

    commands.spawn((
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("main.gltf")),
        ),
        Name::new("World Root main.gltf"),
        Visibility::Visible,
    ));
}

// TODO: i mean we can just despawn any bullets that collide with anything
pub fn detect_bullet_collision_with_wall_and_grounds(
    mut commands: Commands,
    bullet_query: Query<Entity, Or<(With<PlayerBullet>, With<EnemyBullet>)>>,
    wall_and_ground_query: Query<
        (Entity, &Transform),
        Or<(With<Wall>, With<Ground>)>,
    >,
) {
    // for CollisionStarted(first_entity, second_entity) in
    //     collision_event_reader.read()
    // {
    //     let Some(bullet_entity) = bullet_query
    //         .iter()
    //         .find(|e| e == first_entity || e == second_entity)
    //     else {
    //         continue;
    //     };
    //
    //     let collided_entities_is_wall_or_ground =
    //         wall_and_ground_query.iter().any(|(entity, _)| {
    //             entity == *first_entity || entity == *second_entity
    //         });
    //     if !collided_entities_is_wall_or_ground {
    //         continue;
    //     }
    //
    //     commands.entity(bullet_entity).despawn();
    // }
}
