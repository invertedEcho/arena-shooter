use std::time::Instant;

use crate::nav_mesh_pathfinding::{ArchipelagoRef, ENEMY_AGENT_RADIUS};
use avian3d::{math::Quaternion, prelude::*};
use bevy::prelude::*;
use bevy_landmass::{
    Agent, Agent3dBundle, AgentSettings, AgentTarget3d, ArchipelagoRef3d,
};
use shared::{
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS, RUN_VELOCITY,
        WALK_VELOCITY,
        components::{CharacterController, Grounded},
    },
    components::Health,
    enemy::components::{Enemy, EnemyLastStateUpdate, EnemyState},
    game_score::{GameScore, LivingEntityStats},
    protocol::EntityPositionServer,
};

pub struct EnemySpawnPlugin;

impl Plugin for EnemySpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnEnemiesMessage>().add_systems(
            Update,
            (handle_spawn_enemies_at_enemy_spawn_locations_message,),
        );
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EnemySpawnLocation;

#[derive(Message)]
pub struct SpawnEnemiesMessage {
    pub enemy_count: usize,
    pub spawn_strategy: EnemySpawnStrategy,
}

#[derive(Component)]
pub struct AgentEnemyEntityPointer(pub Entity);

pub enum EnemySpawnStrategy {
    /// Enemies will be spawned at randomly picked EnemySpawnLocations
    RandomSelection,
}

fn handle_spawn_enemies_at_enemy_spawn_locations_message(
    mut message_reader: MessageReader<SpawnEnemiesMessage>,
    mut commands: Commands,
    enemy_spawn_locations: Query<
        (Entity, &Transform),
        With<EnemySpawnLocation>,
    >,
    archipelago_ref: Option<Res<ArchipelagoRef>>,
    mut game_score: Single<&mut GameScore>,
) {
    for event in message_reader.read() {
        let Some(ref archipelago_ref) = archipelago_ref else {
            warn!(
                "Received enemy spawn message but archipelago_ref doesnt \
                 exist yet, ignoring message"
            );
            return;
        };

        if enemy_spawn_locations.is_empty() {
            error!("Requested enemy spawn but no spawn locations exist!");
            continue;
        }

        let mut spawn_enemy_count = event.enemy_count;
        let spawn_method = &event.spawn_strategy;

        match spawn_method {
            EnemySpawnStrategy::RandomSelection => {
                if spawn_enemy_count > enemy_spawn_locations.iter().len() {
                    warn!(
                        "Requested more enemy spawns than available \
                         EnemySpawnLocations, decreasing. This will mean \
                         losing enemy spawns"
                    );
                    spawn_enemy_count -= enemy_spawn_locations
                        .iter()
                        .len()
                        .abs_diff(spawn_enemy_count);
                    warn!(
                        "Original enemy spawn count: {} | New \
                         spawn_enemy_count: {}",
                        event.enemy_count, spawn_enemy_count
                    );
                }

                let mut already_used_spawn_locations: Vec<Entity> = Vec::new();
                // i kinda dont like while loops as its very easy to cause infinite loops with them
                while already_used_spawn_locations.len() != spawn_enemy_count {
                    let chosen_spawn_location_index = rand::random_range(
                        0..enemy_spawn_locations.iter().len(),
                    );
                    if already_used_spawn_locations.contains(
                        &enemy_spawn_locations
                            .iter()
                            .collect::<Vec<(Entity, &Transform)>>()
                            [chosen_spawn_location_index]
                            .0,
                    ) {
                        continue;
                    }

                    let chosen_spawn_location = enemy_spawn_locations
                        .iter()
                        .collect::<Vec<(Entity, &Transform)>>()
                        [chosen_spawn_location_index];
                    already_used_spawn_locations.push(chosen_spawn_location.0);

                    let spawn_location_translation =
                        chosen_spawn_location.1.translation;

                    debug!(
                        "Spawning an enemy at {}",
                        spawn_location_translation
                    );

                    let enemy_entity = commands
                        .spawn((
                            Name::new("Enemy"),
                            Transform::from_translation(
                                spawn_location_translation,
                            ),
                            Enemy,
                            EnemyLastStateUpdate(Instant::now()),
                            Health(100.0),
                            EnemyState::default(),
                            Grounded::default(),
                            EntityPositionServer {
                                translation: spawn_location_translation,
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
                            .with_max_distance(0.2),
                            CharacterController,
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
                        Transform::from_xyz(0.0, -0.8, 0.0),
                        AgentEnemyEntityPointer(enemy_entity),
                    ));
                }
            }
        }
    }
}
