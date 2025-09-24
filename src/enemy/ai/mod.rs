use avian3d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};
use vleue_navigator::{NavMesh, prelude::NavMeshSettings};

use crate::{
    enemy::Enemy, game_flow::AppState, nav_mesh_pathfinding::CurrentNavMesh,
    player::Player,
};

// TODO: actually implement this lol
// Enemy AI is currrently working like this:
// A raycast is shot to the direction of the player
//    If the raycast first hit is the player, the enemy state changes to AttackPlayer and will
//    shoot him.
//    If not, it means the enemy can not see the player. Then, we use the pathfinding library
//    together with our navmesh to find the fastest path to the player
//    (in the future we will of course not just take the fastest route, but have some kind of
//    randomness?)

pub struct EnemyAiPlugin;

impl Plugin for EnemyAiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CheckIfEnemyCanSeePlayerCooldownTimer(
            Timer::from_seconds(0.1, TimerMode::Repeating),
        ))
        .add_event::<StartChasingPlayerEvent>()
        .add_systems(
            Update,
            (
                check_if_enemy_can_see_player_and_look_at_player,
                tick_enemy_can_see_player_cooldown_timer,
                handle_start_chasing_player_event,
                enemy_patrol,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Default, Reflect, PartialEq)]
pub enum EnemyState {
    #[default]
    Idle,
    /// Going to the last known location of player
    ChasingPlayer,
    /// Enemy can see the player, will shoot the player now
    AttackPlayer,
    /// This state will be set when the enemy has 0 health. It will just play a death animation and
    /// afterwards be despawned.
    Dead,
}

/// This event will get fired when the enemy can not directly see the player.
/// A system will handle this event, and set current patrol destination and next patrol destinations
#[derive(Event)]
pub struct StartChasingPlayerEvent {
    /// The enemy entity that should start chasing the player
    pub enemy_entity: Entity,
}

#[derive(Resource)]
pub struct CheckIfEnemyCanSeePlayerCooldownTimer(pub Timer);

fn tick_enemy_can_see_player_cooldown_timer(
    mut timer: ResMut<CheckIfEnemyCanSeePlayerCooldownTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());
}

fn check_if_enemy_can_see_player_and_look_at_player(
    spatial_query: SpatialQuery,
    enemy_query: Query<
        (&mut Enemy, Entity, &mut Transform),
        (Without<Player>, With<Enemy>),
    >,
    player_query: Single<(Entity, &Transform), With<Player>>,
    check_if_enemy_can_see_player_cooldown_timer: Res<
        CheckIfEnemyCanSeePlayerCooldownTimer,
    >,
) {
    // TODO: WAAAIT This means enemies can only shoot all at once, not independtly...
    if !check_if_enemy_can_see_player_cooldown_timer
        .0
        .just_finished()
    {
        return;
    }
    let player_entity = player_query.0;
    let player_transform = player_query.1;

    for (mut enemy, enemy_entity, mut enemy_transform) in enemy_query {
        if enemy.state == EnemyState::Dead {
            continue;
        }
        // // TODO: of course, while chasing, once the enemy sees the player, it should start shoot him..
        // if enemy.state == EnemyState::ChasingPlayer {
        //     continue;
        // }
        let enemy_translation = enemy_transform.translation;
        let player_translation = player_transform.translation;

        let origin = enemy_translation;

        // direction towards player
        let vector_not_normalized = player_translation - enemy_translation;
        let direction = Dir3::new(vector_not_normalized).unwrap();

        let max_distance = 1000.0;
        let solid = false;

        // raycast shouldnt hit enemy itself
        let filter = SpatialQueryFilter::default()
            .with_excluded_entities([enemy_entity]);

        if let Some(first_hit) = spatial_query.cast_ray(
            origin,
            direction,
            max_distance,
            solid,
            &filter,
        ) {
            if first_hit.entity == player_entity {
                if enemy.state != EnemyState::AttackPlayer {
                    enemy.state = EnemyState::AttackPlayer;
                }
                enemy_transform.look_at(player_transform.translation, Dir3::Y);
            } else {
                if enemy.state != EnemyState::Idle {
                    enemy.state = EnemyState::Idle;
                }
            }
        }
    }
}

fn handle_start_chasing_player_event(
    mut commands: Commands,
    mut start_chasing_player_event_reader: EventReader<StartChasingPlayerEvent>,
    mut enemy_query: Query<
        (&mut Enemy, Entity, &mut Transform),
        (Without<Player>, With<Enemy>),
    >,
    player_transform: Single<&Transform, With<Player>>,
    navmeshes: Res<Assets<NavMesh>>,
    current_navmesh: Res<CurrentNavMesh>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in start_chasing_player_event_reader.read() {
        let Some(enemy) = enemy_query
            .iter_mut()
            .find(|(_, entity, _)| *entity == event.enemy_entity)
        else {
            warn!("LKJSDFKL:JFDSKL");
            continue;
        };

        let (mut enemy, _, enemy_transform) = enemy;

        let navmesh = navmeshes.get(&current_navmesh.0).unwrap();
        let path = navmesh.transformed_path(
            Vec3 {
                x: enemy_transform.translation.x,
                y: 0.0,
                z: enemy_transform.translation.z,
            },
            Vec3 {
                x: player_transform.translation.x,
                y: 0.0,
                z: player_transform.translation.z,
            },
        );
        match path {
            Some(res) => {
                enemy.state = EnemyState::ChasingPlayer;
                enemy.current_patrol_destination = Some(res.path[0]);
                enemy.next_patrol_destinations = Some(res.path[1..].to_vec());
                for point in res.path {
                    commands.spawn((
                        Transform::from_translation(point),
                        Mesh3d(meshes.add(Sphere::new(0.1))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: RED.into(),
                            ..Default::default()
                        })),
                    ));
                }
            }
            None => {
                warn!("Could not find path from enemy to player");
            }
        }
    }
}

fn enemy_patrol(
    enemy_query: Query<(&mut Enemy, &mut LinearVelocity, &mut Transform)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    nav_mesh_settings: Query<&NavMeshSettings>,
) {
    if !keyboard_input.pressed(KeyCode::KeyH) {
        return;
    }
    for i in nav_mesh_settings {
        info!("WOWO: {:?}", i);
    }

    for (mut enemy, mut velocity, mut transform) in enemy_query {
        let Some(current_patrol_destination) = enemy.current_patrol_destination
        else {
            return;
        };

        info!("enemy transform: {}", transform.translation);
        info!("current patrol point: {}", current_patrol_destination);

        let fixed_enemy = Vec3 {
            x: transform.translation.x,
            y: 0.0,
            z: transform.translation.z,
        };

        info!(
            "distance: {}",
            fixed_enemy.distance(current_patrol_destination)
        );

        let enemy_reached_patrol_point =
            fixed_enemy.distance(current_patrol_destination) < 0.4;

        if enemy_reached_patrol_point {}

        if enemy_reached_patrol_point {
            info!("ENEMY REACHED PATROL POINT!");
            velocity.z = 0.0;

            let Some(next_patrol_destinations) =
                enemy.next_patrol_destinations.clone()
            else {
                info!(
                    "Enemy reached patrol point and next next_patrol_destinations is None"
                );
                // TODO: should probably check if we can see the player now and if not, send
                // StartChasingPlayerEvent.
                continue;
            };

            enemy.current_patrol_destination =
                Some(next_patrol_destinations[0]);
            enemy.next_patrol_destinations =
                Some(next_patrol_destinations[1..].to_vec());
            info!("enemy reached patrol point, updated patrol destinations!");
            transform.look_at(
                enemy
                    .current_patrol_destination
                    .expect("current patrol destination must exist"),
                Vec3::Y,
            );
            info!("enemy now looks at new current_patrol_destination");
            continue;
        };

        let mut local_velocity = Vec3::ZERO;
        local_velocity.z = -2.0;
        let world_velocity = transform.rotation * local_velocity;
        **velocity = world_velocity;
    }
}
