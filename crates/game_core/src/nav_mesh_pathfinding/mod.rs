use avian_rerecast::AvianBackendPlugin;
use bevy::{platform::collections::HashSet, prelude::*};
use bevy_landmass::prelude::*;
use bevy_rerecast::{Navmesh, prelude::*};
use landmass_rerecast::{
    Island3dBundle, LandmassRerecastPlugin, NavMeshHandle3d,
};
use shared::{
    AppRole,
    character_controller::{CHARACTER_HEIGHT, MAX_SLOPE_ANGLE},
    world_object::WorldObjectCollectibleServerSide,
};

use crate::GameCoreLoadingState;

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
            OnEnter(GameCoreLoadingState::CollidersSpawned),
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
    all_entities_except_world_objects: Query<
        Entity,
        Without<WorldObjectCollectibleServerSide>,
    >,
    app_role: Res<State<AppRole>>,
    mut next_server_loading_state: ResMut<NextState<GameCoreLoadingState>>,
) {
    // NOTE: We only do nav mesh generation in case this is ClientAndServer. no use on dedicated
    // server currently
    if *app_role.get() != AppRole::ClientAndServer {
        next_server_loading_state.set(GameCoreLoadingState::Done);
        return;
    }

    info!("Colliders spawned, generating NavMesh");
    let nav_mesh_settings = NavmeshSettings {
        // TODO: document why this radius is smaller than ENEMY_AGENT_RADIUS
        agent_radius: 0.3,
        // this is pretty important, so the agent doesnt try to climb some very high ledge
        walkable_climb: 0.25,
        walkable_slope_angle: MAX_SLOPE_ANGLE,
        cell_size_fraction: 2.0,
        cell_height_fraction: 4.0,
        agent_height: CHARACTER_HEIGHT,
        // exclude world object coliders from nav mesh generation
        filter: Some(HashSet::from_iter(all_entities_except_world_objects)),
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
    mut next_server_loading_state: ResMut<NextState<GameCoreLoadingState>>,
    mut nav_meshes: ResMut<Assets<Navmesh>>,
) {
    let Some(nav_mesh_handle) = nav_meshes.get_strong_handle(trigger.0) else {
        panic!(
            "Got navmeshready event but the Handle could not be found using \
             the asset id from the trigger"
        );
    };

    commands.insert_resource(NavMeshHandle(nav_mesh_handle));

    info!("NavMesh is now ready, updating ServerLoadingState to Done");
    next_server_loading_state.set(GameCoreLoadingState::Done);
}

// fn log_agent_state(agent_state: Query<&AgentState>) {
//     for agent_state in agent_state {
//         info!("Agent state: {:?}", agent_state);
//     }
// }
