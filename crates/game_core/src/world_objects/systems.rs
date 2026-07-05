use avian3d::prelude::*;
use bevy::prelude::*;
use netvy::prelude::*;
use shared::{
    AppRole, DEFAULT_HEALTH, GameConfigServer, GameMap,
    components::Health,
    player::{Player, PlayerState},
    shooting::PlayerWeapons,
    world_object::{
        WorldObjectCollectibleKind, WorldObjectCollectibleServerSide,
    },
};

use crate::{
    SpawnLocationFile,
    world_objects::{DEFAULT_HEALTH_TO_GIVE_MEDKIT, components::RespawnTimer},
};

#[derive(Resource)]
pub struct CurrentSpawnLocationsHandle(Handle<SpawnLocationFile>);

// FIXME: this shouldnt happen just as a random side effect. why exactly do we need to do this?
// I think we did it here just as a dirty fix, when user selects a map, we already pre load the
// spawn_locations.json, so once the match starts, the handle is already ready.
pub fn load_spawn_locations(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    game_config: Res<GameConfigServer>,
) {
    let file_path = match game_config.0.game_map {
        GameMap::MediumPlastic => "maps/medium_plastic/spawn_locations.json",
        GameMap::TinyTown => "maps/tiny_town/spawn_locations.json",
    };
    debug!("Loading spawn location file: {}", file_path);

    let handle = asset_server.load(file_path);
    debug!("Inserting SpawnLocationFile handle {:?}", handle.id());
    commands.insert_resource(CurrentSpawnLocationsHandle(handle));
}

pub fn spawn_world_objects(
    mut commands: Commands,
    app_role: Res<State<AppRole>>,
    current_spawn_location_handle: Option<Res<CurrentSpawnLocationsHandle>>,
    spawn_locations: ResMut<Assets<SpawnLocationFile>>,
) {
    let Some(current_spawn_location_handle) = current_spawn_location_handle
    else {
        error!(
            "CurrentSpawnLocationsHandle must exist in order to be able to \
             spawn WorldObjects (such as medkits)"
        );
        return;
    };
    if *app_role.get() == AppRole::ClientOnly {
        info!(
            "Not spawning WorldObjectCollectibleServerSide, this is ClientOnly"
        );
        return;
    }

    let Some(spawn_location) =
        spawn_locations.get(current_spawn_location_handle.0.id())
    else {
        error!(
            "Failed to load spawn locations, no SpawnLocations will be \
             spawned. (spawn_location_handle={})",
            current_spawn_location_handle.0.id()
        );
        return;
        // panic!(
        //     "Failed to load spawn locations, the asset hasnt been loaded yet \
        //      or resource doesnt exist yet. Handle we wanted to retrieve: {}",
        //     current_spawn_location_handle.0.id()
        // );
    };

    info!("Loaded spawn locations for world objects, spawning...");

    for spawn_location in &spawn_location.positions {
        commands.spawn((
            Transform::from_translation(spawn_location.position),
            ReplicateEntity,
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
