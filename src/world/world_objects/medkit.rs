use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    game_flow::states::AppState,
    player::{DEFAULT_PLAYER_HEALTH, Player},
    shared::components::Health,
};

const MEDKIT_MODEL_PATH: &str = "models/world_objects/medkit.gltf";

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MedkitSpawnLocation;

const DEFAULT_HEALTH_TO_GIVE_MEDKIT: f32 = 25.0;

#[derive(Component)]
pub struct Medkit {
    active: bool,
    health_to_give: f32,
    float_direction: FloatDirection,
    respawn_timer: Timer,
}

enum FloatDirection {
    Up,
    Down,
}

pub fn spawn_medkits(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    medkit_spawn_locations: Query<Entity, With<MedkitSpawnLocation>>,
) {
    if medkit_spawn_locations.is_empty() {
        warn!("no medkit spawn locations");
    }

    for entity in medkit_spawn_locations {
        let medkit_model = asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(MEDKIT_MODEL_PATH));

        // we insert the medkit itself as a child of the spawn location because it makes sense
        // sementically, but also because then we dont need to save the origin transform for
        // floating up and down, and just use 0.0 as origin_y
        commands.entity(entity).with_child((
            DespawnOnExit(AppState::InGame),
            SceneRoot(medkit_model),
            Collider::cuboid(0.2, 0.2, 0.2),
            Medkit {
                float_direction: FloatDirection::Down,
                health_to_give: DEFAULT_HEALTH_TO_GIVE_MEDKIT,
                active: true,
                respawn_timer: Timer::new(
                    Duration::from_secs(5),
                    TimerMode::Repeating,
                ),
            },
            Name::new("Medkit"),
            CollidingEntities::default(),
            Visibility::Visible,
        ));
    }
}

pub fn rotate_and_float_medkits(
    medkits_query: Query<(&mut Medkit, &mut Transform), With<Medkit>>,
    time: Res<Time>,
) {
    const ORIGIN_Y: f32 = 0.0;
    for (mut medkit, mut medkit_transform) in medkits_query {
        medkit_transform.rotate_y(1. * time.delta_secs());

        let current_y = medkit_transform.translation.y;

        match medkit.float_direction {
            FloatDirection::Down => {
                medkit_transform.translation.y -= 0.2 * time.delta_secs();
                if ORIGIN_Y - current_y > 0.1 {
                    medkit.float_direction = FloatDirection::Up;
                }
            }
            FloatDirection::Up => {
                medkit_transform.translation.y += 0.2 * time.delta_secs();
                if current_y - ORIGIN_Y > 0.1 {
                    medkit.float_direction = FloatDirection::Down;
                }
            }
        }
    }
}

pub fn detect_collision_medkit_with_player(
    medkit_query: Query<(&mut Medkit, &CollidingEntities, &mut Visibility)>,
    mut player_query: Single<(Entity, &mut Health), With<Player>>,
) {
    let (player_entity, mut player_health) = player_query.into_inner();

    let player_full_hp = player_health.0 == DEFAULT_PLAYER_HEALTH;

    for (mut medkit, colliding_entities, mut visibility) in medkit_query {
        if !medkit.active {
            continue;
        }

        let player_collied_with_medkit =
            colliding_entities.contains(&player_entity);

        if player_collied_with_medkit && !player_full_hp {
            player_health.0 += medkit.health_to_give;
            player_health.0 = player_health.0.clamp(0.0, DEFAULT_PLAYER_HEALTH);
            medkit.active = false;
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn activate_medkits_over_time(
    medkit_query: Query<(&mut Medkit, &mut Visibility)>,
) {
    for (mut medkit, mut visibility) in medkit_query {
        if !medkit.active && medkit.respawn_timer.is_finished() {
            medkit.active = true;
            *visibility = Visibility::Visible;
        }
    }
}

pub fn tick_respawn_timer_medkits(
    medkits_query: Query<&mut Medkit>,
    time: Res<Time>,
) {
    for mut medkit in medkits_query {
        if !medkit.active {
            medkit.respawn_timer.tick(time.delta());
        }
    }
}
