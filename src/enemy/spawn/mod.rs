use crate::enemy::{
    animate::SWAT_MODEL_PATH, shooting::EnemyShootPlayerCooldownTimer,
};
use avian3d::{math::PI, prelude::*};
use bevy::prelude::*;

use crate::enemy::Enemy;

pub struct EnemySpawnPlugin;

impl Plugin for EnemySpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_enemies_at_enemy_spawn_locations,
                tick_enemy_spawn_timer,
            ),
        )
        .insert_resource(EnemySpawnTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )));
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EnemySpawnLocation;

#[derive(Resource)]
struct EnemySpawnTimer(pub Timer);

fn tick_enemy_spawn_timer(
    mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    time: Res<Time>,
) {
    enemy_spawn_timer.0.tick(time.delta());
}

fn spawn_enemies_at_enemy_spawn_locations(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    enemy_spawn_locations: Query<&Transform, Added<EnemySpawnLocation>>,
) {
    for added_enemy_spawn_location in enemy_spawn_locations {
        let enemy_model = asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(SWAT_MODEL_PATH));

        commands
            .spawn((
                Transform::from_translation(
                    added_enemy_spawn_location.translation,
                ),
                Enemy {
                    health: 100.0,
                    ..default()
                },
                RigidBody::Dynamic,
                LockedAxes::new()
                    .lock_rotation_x()
                    .lock_rotation_y()
                    .lock_rotation_z(),
                Collider::cuboid(0.3, 1.7, 0.3),
                AngularVelocity::ZERO,
                LinearVelocity::ZERO,
                EnemyShootPlayerCooldownTimer(Timer::from_seconds(
                    1.0,
                    TimerMode::Repeating,
                )),
                Visibility::Visible,
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
            ));
    }
}
