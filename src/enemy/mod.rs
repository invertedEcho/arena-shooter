use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{player::Player, world::components::Map};

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

pub fn setup_ray_cast(
    mut commands: Commands,
    enemy_query: Query<&Transform, Added<Enemy>>,
    map_entity: Single<Entity, With<Map>>,
) {
    for enemy_transform in enemy_query {
        info!(
            "Setting up enemy ray cast, which also means player entity was found!"
        );
        info!("Player entity: {}", *map_entity);

        // let query_filter = SpatialQueryFilter::from_mask(0b1011)
        //     .with_excluded_entities([*map_entity]);

        commands.spawn(
            RayCaster::new(enemy_transform.translation, Dir3::X), // .with_query_filter(query_filter),
        );
    }
}

pub fn print_raycast_hits(
    query: Query<(&RayCaster, &RayHits)>,
    player_entity: Single<Entity, With<Player>>,
) {
    for (_, hits) in &query {
        for hit in hits.iter() {
            let entity = hit.entity;

            // TODO: Find out how to do filter raycast. Only `.with_excluded_entities()` exist, but
            // I dont want to mark everything except `Player` with some marker component...
            if entity == *player_entity {
                info!("raycast hit player!");
            }
        }
    }
}
