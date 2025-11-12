use avian_rerecast::AvianBackendPlugin;
use avian3d::prelude::*;
use bevy::{platform::collections::HashSet, prelude::*};
use bevy_landmass::prelude::*;
use bevy_rerecast::prelude::*;
use landmass_rerecast::{
    Island3dBundle, LandmassRerecastPlugin, NavMeshHandle3d,
};

use crate::{
    character_controller::{CHARACTER_HEIGHT, MAX_SLOPE_ANGLE},
    enemy::{Enemy, EnemyState, spawn::AgentEnemyEntityPointer},
    game_flow::states::LoadingGameSubState,
    world::world_objects::medkit::Medkit,
};

pub const ENEMY_AGENT_RADIUS: f32 = 0.4;

pub struct NavMeshPathfindingPlugin;

impl Plugin for NavMeshPathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AvianBackendPlugin::default());
        app.add_plugins(NavmeshPlugins::default());
        app.add_plugins(Landmass3dPlugin::default());
        app.add_plugins(LandmassRerecastPlugin::default());
        // app.add_plugins(Landmass3dDebugPlugin::default());
        app.add_systems(
            OnEnter(LoadingGameSubState::CollidersReady),
            generate_navmesh_on_map_colliders_ready,
        );
        app.add_systems(Update, update_agent_velocity_from_physics_velocity);
    }
}

// We store the NavMesh handle in a resource so we can regenerate the navmesh when needed
#[derive(Resource)]
pub struct NavMeshHandle(pub Handle<Navmesh>);

#[derive(Resource)]
pub struct ArchipelagoRef(pub Entity);

// TODO: dont generate navmesh if game mode is FreeRoam
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
        let archipelago_id = commands
            .spawn(Archipelago3d::new(ArchipelagoOptions::from_agent_radius(
                ENEMY_AGENT_RADIUS,
            )))
            .id();
        commands.insert_resource(ArchipelagoRef(archipelago_id));

        let navmesh = generator.generate(nav_mesh_settings);

        // commands.spawn(DetailNavmeshGizmo::new(&navmesh));

        commands.spawn(Island3dBundle {
            island: Island,
            archipelago_ref: ArchipelagoRef3d::new(archipelago_id),
            nav_mesh: NavMeshHandle3d(navmesh),
        });
    }
}

fn update_agent_velocity_from_physics_velocity(
    mut agent_query: Query<(
        &mut Velocity3d,
        &AgentState,
        &AgentEnemyEntityPointer,
    )>,
    mut enemy_query: Query<(&LinearVelocity, &mut Enemy)>,
) {
    for (mut agent_velocity, agent_state, agent_enemy_entity_pointer) in
        agent_query.iter_mut()
    {
        let Ok((enemy_velocity, mut enemy)) =
            enemy_query.get_mut(agent_enemy_entity_pointer.0)
        else {
            warn!(
                "Couldn't find enemy with LinearVelocity by id {}",
                agent_enemy_entity_pointer.0
            );
            continue;
        };
        if *agent_state == AgentState::TargetNotOnNavMesh {
            // FIXME: if the player is somewhere the enemy cant reach,
            // the enemy will never be able to get to the player.
            // we should just try nearby locations instead

            // if the target is not on the navmesh, we let our systems make a new Target, until it
            // is on the navmesh again.
            enemy.state = EnemyState::CheckIfPlayerSeeable;
        }

        agent_velocity.velocity = enemy_velocity.0;
    }
}
