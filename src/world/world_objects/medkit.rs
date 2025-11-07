use std::time::Duration;

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
    active: bool,
    health_to_give: f32,
    float_direction: FloatDirection,
    medkit_spawn_location_parent: Entity,
    respawn_timer: Timer,
}

enum FloatDirection {
    Up,
    Down,
}

pub fn spawn_medkits(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    medkit_spawn_locations: Query<
        (Entity, &Transform),
        With<MedkitSpawnLocation>,
    >,
) {
    if medkit_spawn_locations.is_empty() {
        error!("no medkit spawn locations");
    }

    for (entity, transform) in medkit_spawn_locations {
        info!("Spawning medkit at: {}", transform.translation);
        let medkit_model = asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("medkit.gltf#Scene0"));

        commands.entity(entity).with_child((
            DespawnOnExit(AppState::InGame),
            SceneRoot(medkit_model),
            Collider::cuboid(0.1, 0.1, 0.1),
            RigidBody::Static,
            Medkit {
                float_direction: FloatDirection::Down,
                health_to_give: DEFAULT_HEALTH_TO_GIVE_MEDKIT,
                active: true,
                medkit_spawn_location_parent: entity,
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
    for (mut medkit, mut medkit_transform) in medkits_query {
        medkit_transform.rotate_y(1. * time.delta_secs());

        let current_y = medkit_transform.translation.y;
        let origin_y = 0.0;

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
    medkit_query: Query<(
        Entity,
        &mut Medkit,
        &CollidingEntities,
        &mut Visibility,
    )>,
    mut player_query: Single<(Entity, &mut Player)>,
) {
    for (medkit_entity, mut medkit, colliding_entities, mut visibility) in
        medkit_query
    {
        if !medkit.active {
            continue;
        }
        if !colliding_entities.is_empty() {
            info!("colliding entities: {:?}", colliding_entities);
            info!("player entity: {}", player_query.0);
        }

        let colliding_entity_is_player =
            colliding_entities.contains(&player_query.0);
        let player_full_hp = player_query.1.health == DEFAULT_PLAYER_HEALTH;

        if colliding_entity_is_player && !player_full_hp {
            player_query.1.health += medkit.health_to_give;
            player_query.1.health =
                player_query.1.health.clamp(0.0, DEFAULT_PLAYER_HEALTH);
            medkit.active = false;
            *visibility = Visibility::Hidden;
            commands.entity(medkit_entity).insert(ColliderDisabled);
            info!(
                "Player collided with medkit, hiding medkit and inserting \
                 colliderdisabled"
            );
        }
    }
}

pub fn activate_medkits_over_time(
    mut commands: Commands,
    medkit_query: Query<(Entity, &mut Medkit, &mut Visibility)>,
) {
    for (entity, mut medkit, mut visibility) in medkit_query {
        if !medkit.active && medkit.respawn_timer.is_finished() {
            medkit.active = true;
            commands.entity(entity).remove::<ColliderDisabled>();
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
