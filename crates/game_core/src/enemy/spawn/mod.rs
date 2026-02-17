use std::time::Instant;

use crate::nav_mesh_pathfinding::{ArchipelagoRef, ENEMY_AGENT_RADIUS};
use avian3d::{math::Quaternion, prelude::*};
use bevy::prelude::*;
use bevy_landmass::{
    Agent, Agent3dBundle, AgentSettings, AgentTarget3d, ArchipelagoRef3d,
};
use shared::{
    DEFAULT_HEALTH, SPAWN_POINT_MEDIUM_PLASTIC_MAP,
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS, CHARACTER_FEET,
        MAX_DISTANCE_GROUND_SHAPE_CAST, RUN_VELOCITY, WALK_VELOCITY,
        components::{Grounded, KinematicEntity},
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

#[derive(Message)]
pub struct SpawnEnemiesMessage {
    pub enemy_count: usize,
}

/// Inserted into the pathfinding agent of an enemy, pointing towards the enemy entity that its
/// inserted into
#[derive(Component)]
pub struct EnemyAgentEntityPointer(pub Entity);

fn handle_spawn_enemies_at_enemy_spawn_locations_message(
    mut message_reader: MessageReader<SpawnEnemiesMessage>,
    mut commands: Commands,
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

        let spawn_enemy_count = event.enemy_count;
        let spawn_location_translation = SPAWN_POINT_MEDIUM_PLASTIC_MAP;

        for _ in 0..spawn_enemy_count {
            debug!("Spawning an enemy at {}", spawn_location_translation);

            let enemy_entity = commands
                .spawn((
                    Name::new("Enemy"),
                    Transform::from_translation(spawn_location_translation),
                    Enemy,
                    EnemyLastStateUpdate(Instant::now()),
                    Health(DEFAULT_HEALTH),
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
                    KinematicEntity,
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
                    .with_max_distance(MAX_DISTANCE_GROUND_SHAPE_CAST),
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
                    archipelago_ref: ArchipelagoRef3d::new(archipelago_ref.0),
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
