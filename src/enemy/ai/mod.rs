use avian3d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};
use vleue_navigator::NavMesh;

use crate::{
    enemy::Enemy,
    game_flow::states::{AppState, InGameState},
    nav_mesh_pathfinding::CurrentNavMesh,
    player::{
        Player,
        spawn::{PLAYER_CAPSULE_LENGTH, PLAYER_CAPSULE_RADIUS},
    },
};

/// Enemy AI is currrently working like this:
/// - A raycast is shot to the direction of the player
///    - If the raycast first hit is the player, the enemy state changes to AttackPlayer and will
///      shoot him.
///    - If not, it means the enemy can not see the player. Then, we use the pathfinding library
///      together with our navmesh to find the fastest path to the player
///      (in the future we will of course not just take the fastest route, but have some kind of
///      randomness?)
pub struct EnemyAiPlugin;

impl Plugin for EnemyAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartChasingPlayerEvent>().add_systems(
            Update,
            (
                handle_start_chasing_player_event,
                enemy_patrol,
                handle_enemy_state_changed,
                set_current_enemy_state,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Default, Reflect, PartialEq, Debug)]
pub enum EnemyState {
    #[default]
    Idle,
    /// Going to the location of the player
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

// TODO: we should just use Vec2 as path finding is only supported on 2d anyways
#[derive(Component)]
pub struct EnemyPatrolPath {
    current_destination: Vec3,
    next_destinations: Vec<Vec3>,
}

fn set_current_enemy_state(
    enemy_query: Query<(&mut Enemy, Entity, &Transform)>,
    spatial_query: SpatialQuery,
    player_query: Single<(Entity, &Transform), With<Player>>,
) {
    let (player_entity, player_transform) = *player_query;
    for (mut enemy, enemy_entity, enemy_transform) in enemy_query {
        // check if we can see the player
        // direction towards player
        let vector_not_normalized =
            player_transform.translation - enemy_transform.translation;
        let direction_normalized = Dir3::new(vector_not_normalized).unwrap();

        let max_distance = 100.0;
        let solid = false;

        // raycast shouldnt hit enemy itself
        let filter = SpatialQueryFilter::default()
            .with_excluded_entities([enemy_entity]);
        if let Some(first_hit) = spatial_query.cast_ray(
            enemy_transform.translation,
            direction_normalized,
            max_distance,
            solid,
            &filter,
        ) {
            let enemy_can_see_player = first_hit.entity == player_entity;
            if enemy_can_see_player {
                if enemy.state != EnemyState::AttackPlayer {
                    info!(
                        "Enemy can see player, setting state to AttackPlayer!"
                    );
                    enemy.state = EnemyState::AttackPlayer;
                };
            } else {
                if enemy.state != EnemyState::ChasingPlayer {
                    info!(
                        "Enemy can NOT see player, setting state to ChasingPlayer!"
                    );
                    enemy.state = EnemyState::ChasingPlayer;
                }
            }
        }
    }
}

fn handle_enemy_state_changed(
    changed_enemies: Query<(&Enemy, Entity, &mut Transform), Changed<Enemy>>,
    player_transform: Single<&Transform, (With<Player>, Without<Enemy>)>,
    mut start_chasing_player_event_writer: EventWriter<StartChasingPlayerEvent>,
) {
    for (enemy, enemy_entity, mut enemy_transform) in changed_enemies {
        if enemy.state == EnemyState::AttackPlayer {
            info!(
                "Enemy {} changed state to AttackPlayer, looking at Player, should start shooting player now",
                enemy_entity
            );
            enemy_transform.look_at(player_transform.translation, Vec3::Y);
        } else if enemy.state == EnemyState::ChasingPlayer {
            info!(
                "Enemy {} changed state to chasingplayer, firing StartChasingPlayerEvent",
                enemy_entity
            );
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
        let Some((mut enemy, enemy_entity, mut enemy_transform)) = enemy_query
            .iter_mut()
            .find(|(_, entity, _)| *entity == event.enemy_entity)
        else {
            warn!(
                "A StartChasingPlayerEvent was read, but the enemy entity from the event couldn't be found."
            );
            continue;
        };
        info!(
            "StartChasingPlayerEvent was read for enemy_entity: {}",
            enemy_entity
        );

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
                commands.entity(enemy_entity).insert(EnemyPatrolPath {
                    current_destination: res.path[0],
                    next_destinations: res.path[1..].to_vec(),
                });

                // make the enemy look at the first patrol path
                let current_destination_fixed = Vec3 {
                    x: res.path[0].x,
                    y: enemy_transform.translation.y,
                    z: res.path[0].z,
                };
                enemy_transform.look_at(current_destination_fixed, Vec3::Y);

                for point in res.path {
                    commands.spawn((
                        Transform::from_translation(point),
                        Mesh3d(meshes.add(Sphere::new(0.05))),
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
    current_in_game_state: Res<State<InGameState>>,
) {
    for (
        entity,
        mut enemy,
        mut enemy_patrol_path,
        mut velocity,
        mut enemy_transform,
    ) in enemies_with_patrol_path
    {
        let in_game_state_is_playing =
            *current_in_game_state.get() != InGameState::Playing;
        if !in_game_state_is_playing {
            **velocity = Vec3::ZERO;
            continue;
        }

        if enemy.state != EnemyState::ChasingPlayer {
            continue;
        }

        let fixed_enemy_transform = Vec3 {
            x: enemy_transform.translation.x,
            y: 0.0,
            z: enemy_transform.translation.z,
        };

        let current_distance_from_enemy_to_current_destination =
            fixed_enemy_transform
                .distance(enemy_patrol_path.current_destination);
        let enemy_reached_patrol_path =
            current_distance_from_enemy_to_current_destination < 0.1;

        if enemy_reached_patrol_path {
            info!("Enemy reached current patrol point!");
            **velocity = Vec3::splat(0.0);

            if enemy_patrol_path.next_destinations.len() == 0 {
                enemy.state = EnemyState::Idle;
                continue;
            }

            enemy_patrol_path.current_destination =
                enemy_patrol_path.next_destinations[0];

            enemy_patrol_path.next_destinations =
                enemy_patrol_path.next_destinations[1..].to_vec();

            let current_destination_fixed = Vec3 {
                x: enemy_patrol_path.current_destination.x,
                y: enemy_transform.translation.y,
                z: enemy_patrol_path.current_destination.z,
            };
            enemy_transform.look_at(current_destination_fixed, Vec3::Y);
            continue;
        };

        let mut local_velocity = Vec3::ZERO;
        local_velocity.z -= 2.0;

        let world_velocity = enemy_transform.rotation * local_velocity;
        let Some(normalized_world_velocity) = world_velocity.try_normalize()
        else {
            **velocity = Vec3::splat(0.0);
            return;
        };

        let world_direction = Dir3::new_unchecked(normalized_world_velocity);

        if let Some(_) = spatial_query.cast_shape(
            &Collider::capsule(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_LENGTH),
            enemy_transform.translation,
            enemy_transform.rotation,
            world_direction,
            &ShapeCastConfig {
                max_distance: 0.1,
                ..default()
            },
            &SpatialQueryFilter::default().with_excluded_entities([entity]),
        ) {
            **velocity = Vec3::ZERO;
            return;
        }

        **velocity = world_velocity;
    }
}
