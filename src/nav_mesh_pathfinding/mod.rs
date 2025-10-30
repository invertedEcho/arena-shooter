use avian_rerecast::AvianBackendPlugin;
use bevy::prelude::*;
use bevy_landmass::{debug::Landmass3dDebugPlugin, prelude::*};
use bevy_rerecast::{debug::DetailNavmeshGizmo, prelude::*};
use landmass_rerecast::{
    Island3dBundle, LandmassRerecastPlugin, NavMeshHandle3d,
};

use crate::{
    character_controller::MAX_SLOPE_ANGLE, game_flow::states::GameLoadingState,
};

pub const ENEMY_AGENT_RADIUS: f32 = 0.4;

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
            agent_radius: 0.3,
            // this is pretty important, so the agent doesnt try to climb some very high ledge
            walkable_climb: 0.25,
            walkable_slope_angle: MAX_SLOPE_ANGLE,
            cell_size_fraction: 2.0,
            cell_height_fraction: 4.0,
            agent_height: 2.0,
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
        info!("Agent state: {:?}", agent_state);
        agent_velocity.velocity = desired_velocity.velocity();
    }
}
