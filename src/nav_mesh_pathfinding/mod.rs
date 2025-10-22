use avian_rerecast::AvianBackendPlugin;
use bevy::prelude::*;
use bevy_landmass::prelude::*;
use bevy_rerecast::{debug::DetailNavmeshGizmo, prelude::*};
use landmass_rerecast::{
    Island3dBundle, LandmassRerecastPlugin, NavMeshHandle3d,
};

use crate::game_flow::states::GameLoadingState;

pub struct NavMeshPathfindingPlugin;

impl Plugin for NavMeshPathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AvianBackendPlugin::default());
        app.add_plugins(NavmeshPlugins::default());
        app.add_plugins(Landmass3dPlugin::default());
        app.add_plugins(LandmassRerecastPlugin::default());
        app.add_systems(
            OnEnter(GameLoadingState::WorldLoadedWithDependencies),
            generate_navmesh,
        );
        app.add_systems(Update, check_agents);
    }
}

#[derive(Resource)]
pub struct ArchipelagoRef(pub Entity);

fn generate_navmesh(mut commands: Commands, mut generator: NavmeshGenerator) {
    info!("generate_navmesh system called");
    let archipelago_id = commands
        .spawn(Archipelago3d::new(ArchipelagoOptions::from_agent_radius(
            0.5,
        )))
        .id();
    commands.insert_resource(ArchipelagoRef(archipelago_id));

    let navmesh = generator.generate(NavmeshSettings {
        agent_radius: 0.2,
        ..default()
    });

    commands.spawn(DetailNavmeshGizmo::new(&navmesh));

    commands.spawn(Island3dBundle {
        island: Island,
        archipelago_ref: ArchipelagoRef3d::new(archipelago_id),
        nav_mesh: NavMeshHandle3d(navmesh),
    });
    info!(
        "generated nav mesh and created landmass island for pathfinding with \
         generated nav mesh!"
    )
}

fn check_agents(agents: Query<(&AgentState, &AgentDesiredVelocity3d)>) {
    for agent in agents {
        info!("agentstate {:?}", agent.0);
        info!("Desired velocity: {:?}", agent.1);
    }
}
