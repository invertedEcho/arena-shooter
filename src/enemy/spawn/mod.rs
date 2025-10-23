use crate::{
    enemy::{
        animate::ENEMY_MODEL_PATH,
        shooting::components::EnemyShootPlayerCooldownTimer,
    },
    nav_mesh_pathfinding::ArchipelagoRef,
    player::{
        Player,
        spawn::{PLAYER_CAPSULE_LENGTH, PLAYER_CAPSULE_RADIUS},
    },
};
use avian3d::{math::PI, prelude::*};
use bevy::{color::palettes::css::BLUE, prelude::*};
use bevy_landmass::{
    Agent, Agent3dBundle, AgentSettings, AgentTarget3d, ArchipelagoRef3d,
};
use bevy_rich_text3d::{Text3d, TextAtlas};

use crate::enemy::Enemy;

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

#[derive()]

pub enum EnemySpawnStrategy {
    /// Enemies will be spawned at randomly picked EnemySpawnLocations
    RandomSelection,
}

fn handle_spawn_enemies_at_enemy_spawn_locations_message(
    mut message_reader: MessageReader<SpawnEnemiesMessage>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    enemy_spawn_locations: Query<
        (Entity, &Transform),
        With<EnemySpawnLocation>,
    >,
    archipelago_ref: Option<Res<ArchipelagoRef>>,
    player_transform: Single<&Transform, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
        let spawn_method = &event.spawn_strategy;

        match spawn_method {
            EnemySpawnStrategy::RandomSelection => {
                let mut already_used_spawn_locations: Vec<Entity> = Vec::new();
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

                    let enemy_model = asset_server.load(
                        GltfAssetLabel::Scene(0).from_asset(ENEMY_MODEL_PATH),
                    );

                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.2))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: BLUE.into(),
                            ..Default::default()
                        })),
                        Transform::from_xyz(
                            player_transform.translation.x,
                            0.,
                            player_transform.translation.z,
                        ),
                    ));

                    let spawn_location_translation =
                        chosen_spawn_location.1.translation;
                    let enemy_entity = commands
                        .spawn((
                            Name::new("Enemy"),
                            Transform::from_xyz(
                                spawn_location_translation.x,
                                spawn_location_translation.y,
                                spawn_location_translation.z,
                            ),
                            Enemy {
                                health: 100.0,
                                ..default()
                            },
                            RigidBody::Kinematic,
                            LockedAxes::new()
                                .lock_rotation_x()
                                .lock_rotation_y()
                                .lock_rotation_z(),
                            Collider::capsule(
                                PLAYER_CAPSULE_RADIUS,
                                PLAYER_CAPSULE_LENGTH,
                            ),
                            AngularVelocity::ZERO,
                            LinearVelocity::ZERO,
                            EnemyShootPlayerCooldownTimer(Timer::from_seconds(
                                0.5,
                                TimerMode::Repeating,
                            )),
                            Visibility::Visible,
                            CollidingEntities::default(),
                        ))
                        .with_child((
                            Transform {
                                // we should probably just fix origin in blender instead of manual offset here
                                translation: Vec3::new(0.0, -0.9, 0.0),
                                // same with rotation here
                                rotation: Quat::from_rotation_y(PI),
                                scale: Vec3::splat(0.9),
                            },
                            SceneRoot(enemy_model),
                            Visibility::Visible,
                        ))
                        .id();
                    commands
                        .entity(enemy_entity)
                        .with_child((
                            Name::new("Enemy Pathfinding Agent"),
                            Agent3dBundle {
                                agent: Agent::default(),
                                archipelago_ref: ArchipelagoRef3d::new(
                                    archipelago_ref.0,
                                ),
                                settings: AgentSettings {
                                    desired_speed: 2.0,
                                    max_speed: 2.0,
                                    radius: 0.3,
                                },
                            },
                            AgentTarget3d::Point(Vec3::new(
                                player_transform.translation.x,
                                0.,
                                player_transform.translation.z,
                            )),
                            Transform::from_xyz(0.0, -0.6, 0.0),
                            AgentEnemyEntityPointer(enemy_entity),
                        ))
                        .with_child((
                            Text3d::new(enemy_entity),
                            Transform::from_xyz(0.0, 1.0, 0.0),
                            Mesh3d::default(),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color_texture: Some(
                                    TextAtlas::DEFAULT_IMAGE.clone(),
                                ),
                                alpha_mode: AlphaMode::Blend,
                                ..Default::default()
                            })),
                        ));
                }
            }
        }
    }
}
