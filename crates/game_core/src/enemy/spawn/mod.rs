use std::time::Instant;

use crate::nav_mesh_pathfinding::{ArchipelagoRef, ENEMY_AGENT_RADIUS};
use avian3d::{math::Quaternion, prelude::*};
use bevy::{color::palettes::css::GREEN, prelude::*};
use bevy_landmass::{
    Agent, Agent3dBundle, AgentSettings, AgentTarget3d, ArchipelagoRef3d,
};
use rand::Rng;
use shared::{
    DEFAULT_HEALTH, SpawnDebugSphereMessage,
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS, CHARACTER_FEET,
        MAX_DISTANCE_GROUNDED_SHAPE_CAST, RUN_VELOCITY, WALK_VELOCITY,
        components::Grounded,
    },
    components::Health,
    enemy::components::{Enemy, EnemyLastStateUpdate, EnemyState},
    game_score::{GameScore, LivingEntityStats},
    protocol::EntityPositionServer,
};

/// A marker component inserted on entities with a mesh, indicating that an enemy may be spawned
/// here.
/// Right now we use this for enemy spawning, specifically getting a random enemy spawn location.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ValidEnemySpawnLocationArea;

pub struct EnemySpawnPlugin;

impl Plugin for EnemySpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnEnemiesMessage>()
            .add_systems(Update, handle_spawn_enemies_message);
    }
}

#[derive(Message)]
pub struct SpawnEnemiesMessage {
    pub enemy_count: usize,
    pub spawn_strategy: EnemySpawnStrategy,
}

/// Inserted into the pathfinding agent of an enemy, pointing towards the enemy entity that its
/// inserted into
#[derive(Component)]
pub struct EnemyAgentEntityPointer(pub Entity);

pub enum EnemySpawnStrategy {
    /// Enemies will be spawned at randomly picked EnemySpawnLocations
    RandomSelection,
}

// first corner is at: 10, 7, 20
// second corner is at: -10, 7, 20
// third corner is at -10, 7, -20
// third corner is at 10, 7, -20

fn get_random_enemy_spawn_locations(
    enemy_spawn_count: usize,
    spatial_query: &mut SpatialQuery,
    valid_enemy_spawn_areas: Vec<Entity>,
) -> Vec<Vec3> {
    const Y: f32 = 7.0;
    let mut rng = rand::rng();

    let mut enemy_spawn_locations: Vec<Vec3> = vec![];

    while enemy_spawn_locations.len() < enemy_spawn_count {
        let random_x = rng.random_range(-10.0..10.0);
        let random_z = rng.random_range(-20.0..20.0);
        let hit = spatial_query.cast_shape(
            &Collider::capsule(0.5, CHARACTER_CAPSULE_LENGTH),
            vec3(random_x, Y, random_z),
            Quat::IDENTITY,
            Dir3::NEG_Y,
            // max distance is just something a bit higher than Y
            &ShapeCastConfig::default().with_max_distance(10.0),
            &SpatialQueryFilter::default(),
        );
        if let Some(hit) = hit
            && valid_enemy_spawn_areas.contains(&hit.entity)
        {
            let mut enemy_spawn_location = hit.point1;
            // elevate the y coordinate so they dont get spawned exactly at hit point, which would
            // be surface of the collider
            enemy_spawn_location.y += CHARACTER_FEET.abs() + 0.5;
            info!(
                "Found valid spawn point thats on a \
                 ValidEnemySpawnLocationArea at {}",
                enemy_spawn_location
            );
            enemy_spawn_locations.push(enemy_spawn_location);
        }
    }
    enemy_spawn_locations
}

fn handle_spawn_enemies_message(
    mut message_reader: MessageReader<SpawnEnemiesMessage>,
    mut commands: Commands,
    archipelago_ref: Option<Res<ArchipelagoRef>>,
    mut game_score: Single<&mut GameScore>,
    mut spatial_query: SpatialQuery,
    mut spawn_debug_sphere_message_writer: MessageWriter<
        SpawnDebugSphereMessage,
    >,
    valid_spawn_location_areas: Query<
        Entity,
        With<ValidEnemySpawnLocationArea>,
    >,
) {
    for event in message_reader.read() {
        let Some(ref archipelago_ref) = archipelago_ref else {
            warn!(
                "Received enemy spawn message but archipelago_ref doesnt \
                 exist yet, ignoring message"
            );
            return;
        };

        let enemy_spawn_count = event.enemy_count;
        let spawn_method = &event.spawn_strategy;

        match spawn_method {
            EnemySpawnStrategy::RandomSelection => {
                let enemy_spawn_locations = get_random_enemy_spawn_locations(
                    enemy_spawn_count,
                    &mut spatial_query,
                    valid_spawn_location_areas.iter().collect(),
                );

                for enemy_spawn_location in enemy_spawn_locations {
                    info!("Spawning an enemy at {}", enemy_spawn_location);
                    spawn_debug_sphere_message_writer.write(
                        SpawnDebugSphereMessage::_new(
                            enemy_spawn_location,
                            GREEN,
                            0.5,
                        ),
                    );

                    let enemy_entity = commands
                        .spawn((
                            Name::new("Enemy"),
                            Transform::from_translation(enemy_spawn_location),
                            Enemy,
                            EnemyLastStateUpdate(Instant::now()),
                            Health(DEFAULT_HEALTH),
                            EnemyState::default(),
                            Grounded::default(),
                            EntityPositionServer {
                                translation: enemy_spawn_location,
                            },
                            RigidBody::Kinematic,
                            Collider::capsule(
                                CHARACTER_CAPSULE_RADIUS,
                                CHARACTER_CAPSULE_LENGTH,
                            ),
                            Visibility::Visible,
                            LinearVelocity::ZERO,
                            CollidingEntities::default(),
                            ShapeCaster::new(
                                Collider::capsule(
                                    CHARACTER_CAPSULE_RADIUS,
                                    CHARACTER_CAPSULE_LENGTH,
                                ),
                                Vec3::ZERO,
                                Quaternion::default(),
                                Dir3::NEG_Y,
                            )
                            .with_max_distance(
                                MAX_DISTANCE_GROUNDED_SHAPE_CAST,
                            ),
                        ))
                        .id();

                    game_score.enemies.insert(
                        enemy_entity,
                        LivingEntityStats {
                            username: format!("Enemy {}", enemy_entity),
                            ..default()
                        },
                    );

                    commands.entity(enemy_entity).with_child((
                        Name::new("Enemy Pathfinding Agent"),
                        Agent3dBundle {
                            agent: Agent::default(),
                            archipelago_ref: ArchipelagoRef3d::new(
                                archipelago_ref.0,
                            ),
                            settings: AgentSettings {
                                desired_speed: WALK_VELOCITY,
                                max_speed: RUN_VELOCITY,
                                radius: ENEMY_AGENT_RADIUS,
                            },
                        },
                        AgentTarget3d::None,
                        // the pathfinding agent must be exacly at the feet of the collider
                        Transform::from_xyz(0.0, CHARACTER_FEET, 0.0),
                        EnemyAgentEntityPointer(enemy_entity),
                    ));
                }
            }
        }
    }
}
