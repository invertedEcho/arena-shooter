use avian3d::prelude::*;
use bevy::prelude::*;

use crate::enemy::{Enemy, EnemySpawnLocation};

const MAX_ENEMY_COUNT: usize = 4;

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
    mut commands: Commands,
    enemy_spawn_timer: Res<EnemySpawnTimer>,
    current_enemies: Query<Entity, With<Enemy>>,
    enemy_spawn_locations: Query<&Transform, With<EnemySpawnLocation>>,
) {
    if current_enemies.iter().len() > MAX_ENEMY_COUNT {
        return;
    }

    if !enemy_spawn_timer.0.just_finished() {
        return;
    }

    let enemy_spawn_location_count = enemy_spawn_locations.iter().len();

    let random_index = rand::random_range(0..enemy_spawn_location_count);

    let random_enemy_spawn_location =
        enemy_spawn_locations.iter().collect::<Vec<&Transform>>()[random_index];

    commands.spawn((
        Transform::from_translation(random_enemy_spawn_location.translation),
        Enemy::default(),
        RigidBody::Static,
        Collider::cuboid(0.3, 1.7, 0.3),
        AngularVelocity::ZERO,
        LinearVelocity::ZERO,
    ));
}
