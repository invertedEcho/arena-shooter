use avian_rerecast::AvianBackendPlugin;
use bevy::prelude::*;
use bevy_landmass::{debug::Landmass3dDebugPlugin, prelude::*};
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
        app.add_plugins(Landmass3dDebugPlugin::default());
        app.add_systems(
            OnEnter(GameLoadingState::CollidersReady),
            generate_navmesh_when_map_colliders_ready,
        );
        app.add_systems(Update, update_agent_velocity);
    }
}

#[derive(Resource)]
pub struct ArchipelagoRef(pub Entity);

fn generate_navmesh_when_map_colliders_ready(
    mut commands: Commands,
    mut generator: NavmeshGenerator,
) {
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

fn update_agent_velocity(
    mut agent_query: Query<(&mut Velocity3d, &AgentDesiredVelocity3d)>,
) {
    for (mut velocity, desired_velocity) in agent_query.iter_mut() {
        velocity.velocity = desired_velocity.velocity();
    }
}
