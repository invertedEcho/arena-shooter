use avian_rerecast::AvianBackendPlugin;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{Agent3d, debug::Landmass3dDebugPlugin, prelude::*};
use bevy_rerecast::{debug::DetailNavmeshGizmo, prelude::*};
use landmass_rerecast::{
    Island3dBundle, LandmassRerecastPlugin, NavMeshHandle3d,
};

use crate::{
    character_controller::MAX_SLOPE_ANGLE,
    enemy::{Enemy, spawn::AgentEnemyEntityPointer},
    game_flow::states::GameLoadingState,
};

pub const ENEMY_AGENT_RADIUS: f32 = 0.3;

pub struct NavMeshPathfindingPlugin;

impl Plugin for NavMeshPathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AvianBackendPlugin::default());
        app.add_plugins(NavmeshPlugins::default());
        app.add_plugins(Landmass3dPlugin::default());
        app.add_plugins(LandmassRerecastPlugin::default());
        app.add_plugins(Landmass3dDebugPlugin::default());
        app.add_systems(
            OnEnter(GameLoadingState::CollidersReady),
            generate_navmesh_when_map_colliders_ready,
        );
        app.add_systems(Update, (update_agent_velocity, snap_agent_to_floor));
    }
}

#[derive(Resource)]
pub struct NavMeshHandle(pub Handle<Navmesh>);

#[derive(Resource)]
pub struct ArchipelagoRef(pub Entity);

fn generate_navmesh_when_map_colliders_ready(
    mut commands: Commands,
    mut generator: NavmeshGenerator,
    maybe_existing_nav_mesh: Option<Res<NavMeshHandle>>,
) {
    let nav_mesh_settings = NavmeshSettings {
        agent_radius: ENEMY_AGENT_RADIUS,
        ..default()
    };

    if let Some(existing_nav_mesh) = maybe_existing_nav_mesh {
        info!("Nav mesh already exists, regenerating!");
        generator.regenerate(&existing_nav_mesh.0, nav_mesh_settings);
    } else {
        let archipelago_id = commands
            .spawn(Archipelago3d::new(ArchipelagoOptions::from_agent_radius(
                ENEMY_AGENT_RADIUS,
            )))
            .id();
        commands.insert_resource(ArchipelagoRef(archipelago_id));

        let navmesh = generator.generate(NavmeshSettings {
            agent_radius: ENEMY_AGENT_RADIUS,
            ..default()
        });

        commands.spawn(DetailNavmeshGizmo::new(&navmesh));

        commands.spawn(Island3dBundle {
            island: Island,
            archipelago_ref: ArchipelagoRef3d::new(archipelago_id),
            nav_mesh: NavMeshHandle3d(navmesh),
        });
    }
}

fn update_agent_velocity(
    mut agent_query: Query<(
        &mut Velocity3d,
        &AgentDesiredVelocity3d,
        &AgentState,
    )>,
) {
    for (mut agent_velocity, desired_velocity, agent_state) in
        agent_query.iter_mut()
    {
        info!("Agent state: {:?}", agent_state,);
        agent_velocity.velocity = desired_velocity.velocity();
    }
}

// FIXME: dont need this anymore
fn snap_agent_to_floor(
    query: Query<
        (Entity, &Transform, &mut LinearVelocity, &ShapeHits),
        With<Enemy>,
    >,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    for (enemy_entity, enemy_transform, mut enemy_velocity, shape_hits) in query
    {
        let Some(first_hit) = shape_hits.0.get(0) else {
            info!("snap_agent_to_floor: No hits down found");
            continue;
        };
        info!("first_hit: {:?}", first_hit);

        // let hit_down_point =
        //     ray_down_origin + ray_down_direction * hit_down.distance;
        // let hit_down_y = hit_down_point.y;
        // let enemy_y = enemy_transform.translation.y;
        // let difference_y = hit_down_y - enemy_y;
        // if difference_y.abs() < 0.3 {
        //     info!("Snapping enemy to slope");
        //     enemy_velocity.y = difference_y / time.delta_secs();
        // } else {
        //     info!(
        //         "Not snapping enemy, difference between hit down and player \
        //          too large"
        //     );
        // }
        // let ray_down_origin = enemy_transform.translation + Vec3::Y * 0.5;
        // let ray_down_direction = Dir3::NEG_Y;
        // let max_down_distance = 1.0;
        //
        // if let Some(hit_down) = spatial_query.cast_ray(
        //     ray_down_origin,
        //     ray_down_direction,
        //     max_down_distance,
        //     true,
        //     &SpatialQueryFilter::default()
        //         .with_excluded_entities([enemy_entity]),
        // ) {
        // } else {
        // }
    }
}
