use avian3d::prelude::*;
use bevy::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Enemy>()
            .add_systems(Update, (setup_ray_cast, print_raycast_hits));
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Enemy;

pub fn setup_ray_cast(mut commands: Commands, enemy_query: Query<&Transform, Added<Enemy>>) {
    for enemy_transform in enemy_query {
        commands.spawn(RayCaster::new(enemy_transform.translation, Dir3::X));
    }
}

pub fn print_raycast_hits(query: Query<(&RayCaster, &RayHits)>) {
    for (ray, hits) in &query {
        for hit in hits.iter_sorted() {
            println!(
                "Hit entity {} at {} with normal {}",
                hit.entity,
                ray.origin + *ray.direction * hit.distance,
                hit.normal,
            );
        }
    }
}
