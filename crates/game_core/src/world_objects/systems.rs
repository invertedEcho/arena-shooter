use std::{fs::File, io::Read};

use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use shared::{
    AppRole, DEFAULT_HEALTH, GameMap,
    components::Health,
    player::{Player, PlayerState},
    shooting::PlayerWeapons,
    world_object::{
        WorldObjectCollectibleKind, WorldObjectCollectibleServerSide,
    },
};

use crate::world_objects::{
    DEFAULT_HEALTH_TO_GIVE_MEDKIT, components::RespawnTimer,
};

#[derive(Debug, Serialize, Deserialize)]
struct SpawnLocation {
    kind: WorldObjectCollectibleKind,
    position: Vec3,
}

fn load_spawn_locations(
    game_map: &GameMap,
) -> Result<std::vec::Vec<SpawnLocation>, serde_json::Error> {
    let file_path = match game_map {
        GameMap::MediumPlastic => {
            "assets/maps/medium_plastic/spawn_locations.json"
        }
        GameMap::TinyTown => "assets/maps/tiny_town/spawn_locations.json",
    };

    let mut file_buffer = String::from("");
    let mut collider_file = File::open(file_path)
        .expect("Can open spawn_locations.json for corresponding game map");

    collider_file.read_to_string(&mut file_buffer).unwrap();

    let spawn_locations: Result<Vec<SpawnLocation>, serde_json::error::Error> =
        serde_json::from_str(&file_buffer);

    spawn_locations
}

pub fn spawn_world_objects(
    mut commands: Commands,
    game_map: Res<State<GameMap>>,
    app_role: Res<State<AppRole>>,
) {
    if *app_role.get() == AppRole::ClientOnly {
        info!(
            "Not spawning WorldObjectCollectibleServerSide, this is ClientOnly"
        );
        return;
    }
    let spawn_locations = load_spawn_locations(game_map.get())
        .expect("Couldn't load and parse spawn locations from json file");
    info!("Loaded spawn locations for world objects, spawning...");

    for spawn_location in spawn_locations {
        commands
            .spawn((
                Transform::from_translation(spawn_location.position),
                Replicate::to_clients(NetworkTarget::All),
            ))
            .with_child((
                Collider::cuboid(0.2, 0.2, 0.2),
                WorldObjectCollectibleServerSide {
                    kind: spawn_location.kind,
                    active: true,
                    position: spawn_location.position,
                },
                CollidingEntities::default(),
                RespawnTimer(Timer::from_seconds(5.0, TimerMode::Repeating)),
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

        match world_object.kind {
            WorldObjectCollectibleKind::Medkit => {
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
            WorldObjectCollectibleKind::Ammunition => {
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
