use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    game_flow::states::AppState,
    player::{DEFAULT_PLAYER_HEALTH, Player},
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MedkitSpawnLocation;

const DEFAULT_HEALTH_TO_GIVE_MEDKIT: f32 = 25.0;

#[derive(Component)]
pub struct Medkit {
    health_to_give: f32,
    origin: Vec3,
    float_direction: FloatDirection,
}

enum FloatDirection {
    Up,
    Down,
}

pub fn spawn_medkits(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    medkit_spawn_locations: Query<&Transform, With<MedkitSpawnLocation>>,
) {
    if medkit_spawn_locations.is_empty() {
        error!("no medkit spawn locations");
    }

    for medkit_spawn_location in medkit_spawn_locations {
        info!("spawning medkit");
        let medkit_model = asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("medkit.gltf#Scene0"));
        commands.spawn((
            DespawnOnExit(AppState::InGame),
            Transform::from_translation(medkit_spawn_location.translation),
            SceneRoot(medkit_model),
            Collider::cuboid(0.1, 0.1, 0.1),
            Medkit {
                origin: medkit_spawn_location.translation,
                float_direction: FloatDirection::Down,
                health_to_give: DEFAULT_HEALTH_TO_GIVE_MEDKIT,
            },
            Name::new("Medkit"),
            CollidingEntities::default(),
        ));
    }
}

pub fn rotate_and_float_medkits(
    medkits_query: Query<(&mut Medkit, &mut Transform), With<Medkit>>,
    time: Res<Time>,
) {
    for (mut medkit, mut medkit_transform) in medkits_query {
        medkit_transform.rotate_y(1. * time.delta_secs());

        let current_y = medkit_transform.translation.y;
        let origin_y = medkit.origin.y;

        match medkit.float_direction {
            FloatDirection::Down => {
                medkit_transform.translation.y -= 0.2 * time.delta_secs();
                if origin_y - current_y > 0.1 {
                    medkit.float_direction = FloatDirection::Up;
                }
            }
            FloatDirection::Up => {
                medkit_transform.translation.y += 0.2 * time.delta_secs();
                if current_y - origin_y > 0.1 {
                    medkit.float_direction = FloatDirection::Down;
                }
            }
        }
    }
}

pub fn detect_collision_medkit_with_player(
    mut commands: Commands,
    medkit_query: Query<(Entity, &Medkit, &CollidingEntities)>,
    mut player_query: Single<(Entity, &mut Player)>,
) {
    for (medkit_entity, medkit, colliding_entities) in medkit_query {
        let colliding_entity_is_player =
            colliding_entities.contains(&player_query.0);
        let player_already_full_hp =
            player_query.1.health == DEFAULT_PLAYER_HEALTH;

        if colliding_entity_is_player && !player_already_full_hp {
            player_query.1.health += medkit.health_to_give;
            player_query.1.health =
                player_query.1.health.clamp(0.0, DEFAULT_PLAYER_HEALTH);
            commands.entity(medkit_entity).despawn();
        }
    }
}
