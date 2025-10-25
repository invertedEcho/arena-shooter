use avian_rerecast::AvianBackendPlugin;
use bevy::prelude::*;
use bevy_landmass::prelude::*;
use bevy_rerecast::{debug::DetailNavmeshGizmo, prelude::*};
use landmass_rerecast::{
    Island3dBundle, LandmassRerecastPlugin, NavMeshHandle3d,
};

use crate::game_flow::states::GameLoadingState;

pub const ENEMY_AGENT_RADIUS: f32 = 0.3;

pub struct NavMeshPathfindingPlugin;

impl Plugin for NavMeshPathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AvianBackendPlugin::default());
        app.add_plugins(NavmeshPlugins::default());
        app.add_plugins(Landmass3dPlugin::default());
        app.add_plugins(LandmassRerecastPlugin::default());
        // app.add_plugins(Landmass3dDebugPlugin::default());
        app.add_systems(
            OnEnter(GameLoadingState::WorldLoadedWithDependencies),
            generate_navmesh,
        );
        app.add_systems(Update, update_agent_velocity);
    }
}

#[derive(Resource)]
pub struct ArchipelagoRef(pub Entity);

fn generate_navmesh(
    mut commands: Commands,
    mut generator: NavmeshGenerator,
    query: Query<(Entity, &Name), With<Mesh3d>>,
) {
    for (entity, name) in query.iter() {
        println!("Entity with {:?} has name: {}", entity, name.as_str());
    }

    info!("generate_navmesh system called");
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
    info!(
        "generated nav mesh and created landmass island for pathfinding with \
         generated nav mesh!"
    )
}

fn update_agent_velocity(
    mut agent_query: Query<(&mut Velocity3d, &AgentDesiredVelocity3d)>,
) {
    for (mut velocity, desired_velocity) in agent_query.iter_mut() {
        velocity.velocity = desired_velocity.velocity();
    }
}
