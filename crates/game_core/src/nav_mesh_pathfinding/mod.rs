use avian_rerecast::AvianBackendPlugin;
use avian3d::prelude::{Collider, CollisionLayers, LayerMask};
use bevy::{platform::collections::HashSet, prelude::*};
use bevy_landmass::prelude::*;
use bevy_rerecast::{Navmesh, prelude::*, rerecast::PolygonNavmesh};
use landmass_rerecast::{
    Island3dBundle, LandmassRerecastPlugin, NavMeshHandle3d,
};
use shared::{
    Medkit, NAV_MESH_LAYER_MASK,
    character_controller::{CHARACTER_HEIGHT, MAX_SLOPE_ANGLE},
};

use crate::ServerLoadingState;

pub const ENEMY_AGENT_RADIUS: f32 = 0.4;

// island: a disconnected isolated group of walkable areas
// archipelago: a group of these islands, that are close to each other but still seperated

pub struct NavMeshPathfindingPlugin;

impl Plugin for NavMeshPathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AvianBackendPlugin::default());
        app.add_plugins(NavmeshPlugins::default());
        app.add_plugins(Landmass3dPlugin::default());
        app.add_plugins(LandmassRerecastPlugin::default());
        app.add_systems(
            OnEnter(ServerLoadingState::CollidersSpawned),
            generate_navmesh_on_map_colliders_ready,
        );
        app.add_observer(on_navmesh_ready);
        // app.add_systems(Update, log_agent_state);
    }
}

/// We store the NavMesh handle in a resource so we can regenerate the navmesh when needed
#[derive(Resource)]
pub struct NavMeshHandle(pub Handle<Navmesh>);

#[derive(Resource)]
pub struct ArchipelagoRef(pub Entity);

fn generate_navmesh_on_map_colliders_ready(
    mut commands: Commands,
    mut generator: NavmeshGenerator,
    maybe_existing_nav_mesh: Option<Res<NavMeshHandle>>,
    all_entities_except_medkits: Query<Entity, Without<Medkit>>,
) {
    let nav_mesh_settings = NavmeshSettings {
        // TODO: document why this radius is smaller than ENEMY_AGENT_RADIUS
        agent_radius: 0.3,
        // this is pretty important, so the agent doesnt try to climb some very high ledge
        walkable_climb: 0.25,
        walkable_slope_angle: MAX_SLOPE_ANGLE,
        cell_size_fraction: 2.0,
        cell_height_fraction: 4.0,
        agent_height: CHARACTER_HEIGHT,
        filter: Some(HashSet::from_iter(all_entities_except_medkits)),
        ..default()
    };

    if let Some(existing_nav_mesh) = maybe_existing_nav_mesh {
        info!("Nav mesh already exists, regenerating!");
        generator.regenerate(&existing_nav_mesh.0, nav_mesh_settings);
    } else {
        let archipelago_options: ArchipelagoOptions<ThreeD> =
            ArchipelagoOptions::from_agent_radius(ENEMY_AGENT_RADIUS);

        let archipelago_id =
            commands.spawn(Archipelago3d::new(archipelago_options)).id();

        commands.insert_resource(ArchipelagoRef(archipelago_id));

        let navmesh = generator.generate(nav_mesh_settings);

        commands.spawn(Island3dBundle {
            island: Island,
            archipelago_ref: ArchipelagoRef3d::new(archipelago_id),
            nav_mesh: NavMeshHandle3d(navmesh),
        });
    }
}

fn on_navmesh_ready(
    trigger: On<NavmeshReady>,
    mut commands: Commands,
    mut next_server_loading_state: ResMut<NextState<ServerLoadingState>>,
    mut nav_meshes: ResMut<Assets<Navmesh>>,
) {
    let Some(mut nav_mesh_handle) = nav_meshes.get_strong_handle(trigger.0)
    else {
        panic!(
            "Got navmeshready event but the Handle could not be found using \
             the asset id from the trigger"
        );
    };

    // we spawn a collider for our navmesh, so we can make raycasts that only hit the navmesh.
    // this is useful to spawn enemies on random locations, but only on our navmesh
    if let Some(mesh) = nav_meshes.get(&mut nav_mesh_handle) {
        commands.spawn((
            navmesh_to_collider(&mesh.polygon),
            CollisionLayers::new(NAV_MESH_LAYER_MASK, LayerMask::ALL),
        ));
    }

    commands.insert_resource(NavMeshHandle(nav_mesh_handle));

    info!("NavMesh is now ready, updating ServerLoadingState to Done");
    next_server_loading_state.set(ServerLoadingState::Done);
}

// yeah ill admit i didnt write this code...
// one day ill know enough about this to fully understand this code
fn navmesh_to_collider(navmesh: &PolygonNavmesh) -> Collider {
    // Convert quantized vertices to world space
    let vertices: Vec<Vec3> = navmesh
        .vertices
        .iter()
        .map(|v| Vec3 {
            x: navmesh.aabb.min.x + v.x as f32 * navmesh.cell_size,
            y: navmesh.aabb.min.y + v.y as f32 * navmesh.cell_height,
            z: navmesh.aabb.min.z + v.z as f32 * navmesh.cell_size,
        })
        .collect();

    // Triangulate polygons (fan triangulation from first vertex)
    let mut indices = Vec::new();
    let mut poly_start = 0;
    for _ in 0..navmesh.polygon_count() {
        let poly = &navmesh.polygons[poly_start
            ..poly_start + navmesh.max_vertices_per_polygon as usize];

        // Find actual vertex count (stop at NO_INDEX)
        let vertex_count = poly
            .iter()
            .position(|&i| i == PolygonNavmesh::NO_INDEX)
            .unwrap_or(poly.len());

        // Skip degenerate polygons
        if vertex_count < 3 {
            poly_start += navmesh.max_vertices_per_polygon as usize;
            continue;
        }

        // Fan triangulation: (0, i, i+1) for i in 1..vertex_count-1
        for i in 1..vertex_count - 1 {
            indices.push([poly[0] as u32, poly[i] as u32, poly[i + 1] as u32]);
        }

        poly_start += navmesh.max_vertices_per_polygon as usize;
    }

    Collider::try_trimesh(vertices, indices)
        .expect("Failed to create navmesh collider")
}

// fn log_agent_state(agent_state: Query<&AgentState>) {
//     for agent_state in agent_state {
//         info!("Agent state: {:?}", agent_state);
//     }
// }
