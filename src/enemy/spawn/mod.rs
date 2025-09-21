use avian3d::{math::PI, prelude::*};
use bevy::prelude::*;

use crate::enemy::{
    Enemy, EnemyShootPlayerCooldownTimer, EnemySpawnLocation, SWAT_MODEL_PATH,
};

pub struct EnemySpawnPlugin;

impl Plugin for EnemySpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (random_spawn_enemies, tick_enemy_spawn_timer))
            .insert_resource(EnemySpawnTimer(Timer::from_seconds(
                1.0,
                TimerMode::Repeating,
            )));
    }
}

#[derive(Resource)]
struct EnemySpawnTimer(pub Timer);

fn tick_enemy_spawn_timer(
    mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    time: Res<Time>,
) {
    enemy_spawn_timer.0.tick(time.delta());
}

// randomly spawn enemies over time
// we need to know possible places where enemies can be spawned.
// so we just have some marker components set around the map
// where enemies can spawn, set in blender
fn random_spawn_enemies(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    enemy_spawn_timer: Res<EnemySpawnTimer>,
    current_enemies: Query<Entity, With<Enemy>>,
    enemy_spawn_locations: Query<&Transform, Added<EnemySpawnLocation>>,
) {
    // if current_enemies.iter().len() > MAX_ENEMY_COUNT {
    //     return;
    // }
    //
    // if !enemy_spawn_timer.0.just_finished() {
    //     return;
    // }
    //

    for added_enemy_spawn_location in enemy_spawn_locations {
        let enemy_model = asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(SWAT_MODEL_PATH));
        info!(
            "spawning enemy at: {}",
            added_enemy_spawn_location.translation
        );
        commands
            .spawn((
                Transform::from_translation(
                    added_enemy_spawn_location.translation,
                ),
                Enemy {
                    health: 100.0,
                    ..default()
                },
                RigidBody::Static,
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

    // let enemy_spawn_location_count = enemy_spawn_locations.iter().len();
    //
    // let random_index = rand::random_range(0..enemy_spawn_location_count);
    //
    // let random_enemy_spawn_location =
    //     enemy_spawn_locations.iter().collect::<Vec<&Transform>>()[random_index];
}
