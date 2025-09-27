use avian3d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};
use vleue_navigator::NavMesh;

use crate::{
    enemy::Enemy,
    game_flow::states::AppState,
    nav_mesh_pathfinding::CurrentNavMesh,
    player::{
        Player,
        spawn::{PLAYER_CAPSULE_LENGTH, PLAYER_CAPSULE_RADIUS},
    },
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
        app.add_event::<StartChasingPlayerEvent>().add_systems(
            Update,
            (
                check_if_enemy_can_see_player_and_look_at_player,
                handle_start_chasing_player_event,
                enemy_patrol,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Default, Reflect, PartialEq, Debug)]
pub enum EnemyState {
    #[default]
    Idle,
    /// Going to the last known location of player
    ChasingPlayer,
    /// Enemy can see the player, will shoot the player now
    AttackPlayer,
    /// This state will be set when `enemy.health == 0.0`. A death animation will be played and
    /// afterwards the enemy will be despawned.
    Dead,
}

/// This event will get fired when the enemy can not directly see the player.
/// A system will handle this event, and set current patrol destination and next patrol destinations
#[derive(Event)]
pub struct StartChasingPlayerEvent {
    /// The enemy entity that should start chasing the player
    pub enemy_entity: Entity,
}

#[derive(Component)]
pub struct EnemyPatrolPath {
    current_destination: Vec3,
    next_destinations: Vec<Vec3>,
}

fn check_if_enemy_can_see_player_and_look_at_player(
    spatial_query: SpatialQuery,
    enemy_query: Query<
        (&mut Enemy, Entity, &mut Transform),
        (Without<Player>, With<Enemy>),
    >,
    player_query: Single<(Entity, &Transform), With<Player>>,
    mut start_chasing_player_event_writer: EventWriter<StartChasingPlayerEvent>,
) {
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
                return;
            } else {
            }
        }

        if enemy.state != EnemyState::ChasingPlayer {
            enemy.state = EnemyState::ChasingPlayer;
            start_chasing_player_event_writer
                .write(StartChasingPlayerEvent { enemy_entity });
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
            warn!(
                "A StartChasingPlayerEvent was read, but the enemy entity from the event couldn't be found."
            );
            continue;
        };

        let (mut enemy, enemy_entity, enemy_transform) = enemy;

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

                commands.entity(enemy_entity).insert(EnemyPatrolPath {
                    current_destination: res.path[0],
                    next_destinations: res.path[1..].to_vec(),
                });

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
    enemies_with_patrol_path: Query<(
        Entity,
        &mut Enemy,
        &mut EnemyPatrolPath,
        &mut LinearVelocity,
        &mut Transform,
    )>,
    spatial_query: SpatialQuery,
) {
    for (
        entity,
        mut enemy,
        mut enemy_patrol_path,
        mut velocity,
        mut transform,
    ) in enemies_with_patrol_path
    {
        if enemy.state != EnemyState::ChasingPlayer {
            continue;
        }

        info!("Enemy state: {:?}", enemy.state);
        let fixed_enemy = Vec3 {
            x: transform.translation.x,
            y: 0.0,
            z: transform.translation.z,
        };

        info!(
            "distance from enemy to current patrol destination: {}",
            fixed_enemy.distance(enemy_patrol_path.current_destination)
        );

        let enemy_reached_patrol_point =
            fixed_enemy.distance(enemy_patrol_path.current_destination) < 0.1;

        if enemy_reached_patrol_point {}

        if enemy_reached_patrol_point {
            info!("Enemy reached current patrol point!");
            velocity.z = 0.0;

            if enemy_patrol_path.next_destinations.len() == 0 {
                info!("enemy has done patroling, no more patrol destinations");
                // TODO: check if im correct, in this case,
                // check_if_enemy_can_see_player_and_look_at_player system will try to locate the
                // player if not, start patroling again?
                enemy.state = EnemyState::Idle;
                continue;
            }

            // TODO: use above check to do something
            enemy_patrol_path.current_destination =
                enemy_patrol_path.next_destinations[0];

            enemy_patrol_path.next_destinations =
                enemy_patrol_path.next_destinations[1..].to_vec();

            transform.look_at(enemy_patrol_path.current_destination, Vec3::Y);
            info!("enemy now looks at new current_patrol_destination");

            continue;
        };

        let mut local_velocity = Vec3::ZERO;
        local_velocity.z -= 2.0;

        let world_velocity = transform.rotation * local_velocity;
        let maybe_normalized_world_velocity = world_velocity.try_normalize();
        let Some(normalized_world_velocity) = maybe_normalized_world_velocity
        else {
            velocity.x = 0.0;
            velocity.z = 0.0;
            return;
        };

        let direction_based_on_input =
            Dir3::new_unchecked(normalized_world_velocity);

        if let Some(first_hit) = spatial_query.cast_shape(
            &Collider::capsule(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_LENGTH),
            transform.translation,
            transform.rotation,
            direction_based_on_input,
            &ShapeCastConfig {
                max_distance: 0.5,
                ..default()
            },
            &SpatialQueryFilter::default().with_excluded_entities([entity]),
        ) {
            if first_hit.distance < 0.1 {
                info!("Disallowing enemy movement, obstacle in the way!");
                **velocity = Vec3::ZERO;
                return;
            }
        }

        let mut local_velocity = Vec3::ZERO;
        local_velocity.z = -2.0;
        let world_velocity = transform.rotation * local_velocity;
        **velocity = world_velocity;
    }
}
