use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::*;
use shared::{
    DEFAULT_HEALTH,
    components::Health,
    player::{Player, PlayerState},
    shooting::PlayerWeapons,
    world_object::{
        WorldObjectCollectibleServerSide, WorldObjectCollectibleType,
    },
};

use crate::world_objects::{
    DEFAULT_HEALTH_TO_GIVE_MEDKIT,
    components::{AmmunitionSpawnLocation, MedkitSpawnLocation, RespawnTimer},
};

pub fn spawn_world_objects(
    mut commands: Commands,
    medkit_spawn_locations: Query<Entity, With<MedkitSpawnLocation>>,
    ammunition_spawn_locations: Query<Entity, With<AmmunitionSpawnLocation>>,
) {
    if medkit_spawn_locations.is_empty() {
        warn!("No MedkitSpawnLocations, no medkits will be spawned");
    }
    if ammunition_spawn_locations.is_empty() {
        warn!("No AmmunitionSpawnLocation, no ammunition will be spawned");
    }

    for entity in medkit_spawn_locations {
        debug!("Spawning a medkit for spawn location: {}", entity);
        commands.entity(entity).with_child((
            Collider::cuboid(0.2, 0.2, 0.2),
            WorldObjectCollectibleServerSide {
                object_type: WorldObjectCollectibleType::Medkit,
                active: true,
            },
            Name::new("Medkit Collider"),
            CollidingEntities::default(),
            RespawnTimer(Timer::from_seconds(5.0, TimerMode::Repeating)),
            Replicate::to_clients(NetworkTarget::All),
        ));
    }

    for entity in ammunition_spawn_locations {
        debug!("Spawning a medkit for spawn location: {}", entity);
        commands.entity(entity).with_child((
            Collider::cuboid(0.2, 0.2, 0.2),
            WorldObjectCollectibleServerSide {
                active: true,
                object_type: WorldObjectCollectibleType::Ammunition,
            },
            Name::new("Ammunition Pack"),
            RespawnTimer(Timer::from_seconds(5.0, TimerMode::Repeating)),
            CollidingEntities::default(),
            Replicate::to_clients(NetworkTarget::All),
        ));
    }
}

pub fn detect_collision_world_object_with_player(
    world_objects_query: Query<(
        &mut WorldObjectCollectibleServerSide,
        &CollidingEntities,
    )>,
    mut player_query: Query<
        (Entity, &mut Health, &mut PlayerWeapons, &PlayerState),
        With<Player>,
    >,
) {
    for (mut world_object, colliding_entities) in world_objects_query {
        if !world_object.active {
            continue;
        }

        match world_object.object_type {
            WorldObjectCollectibleType::Medkit => {
                for collided_entity in colliding_entities.iter() {
                    let Ok((player_entity, mut player_health, _, _)) =
                        player_query.get_mut(*collided_entity)
                    else {
                        continue;
                    };
                    let player_full_hp = player_health.0 == DEFAULT_HEALTH;
                    let player_collied_with_medkit =
                        colliding_entities.contains(&player_entity);

                    if player_collied_with_medkit && !player_full_hp {
                        player_health.0 += DEFAULT_HEALTH_TO_GIVE_MEDKIT;
                        player_health.0 =
                            player_health.0.clamp(0.0, DEFAULT_HEALTH);
                        world_object.active = false;
                    }
                }
            }
            WorldObjectCollectibleType::Ammunition => {
                for collided_entity in colliding_entities.iter() {
                    let Ok((_, _, mut player_weapons, player_state)) =
                        player_query.get_mut(*collided_entity)
                    else {
                        continue;
                    };
                    player_weapons.weapons[player_state.active_weapon_slot]
                        .state
                        .carried_ammo += 60;
                    world_object.active = false;
                }
            }
        }
    }
}

pub fn activate_world_objects_over_time(
    medkit_query: Query<(&mut WorldObjectCollectibleServerSide, &RespawnTimer)>,
) {
    for (mut world_object, respawn_timer) in medkit_query {
        if !world_object.active && respawn_timer.0.is_finished() {
            world_object.active = true;
        }
    }
}

pub fn tick_respawn_timer_world_objects(
    world_objects_query: Query<(
        &WorldObjectCollectibleServerSide,
        &mut RespawnTimer,
    )>,
    time: Res<Time>,
) {
    for (world_object, mut respawn_timer) in world_objects_query {
        if !world_object.active {
            respawn_timer.0.tick(time.delta());
        }
    }
}
