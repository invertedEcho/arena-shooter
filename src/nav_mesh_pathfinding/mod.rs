use avian3d::prelude::*;
use bevy::{
    asset::uuid_handle, color::palettes::tailwind::RED_800, prelude::*,
};
use vleue_navigator::prelude::*;

pub struct NavMeshPathfindingPlugin;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Obstacle;

impl Plugin for NavMeshPathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VleueNavigatorPlugin);
        app.add_plugins(NavmeshUpdaterPlugin::<Collider, Obstacle>::default());
        app.add_systems(Startup, setup)
            .add_systems(Update, view_navmesh);
        app.register_type::<Obstacle>();
    }
}

const NAVMESH_HANDLE: Handle<NavMesh> =
    uuid_handle!("100AD183-2C5C-49A1-AB32-142000E87828");

#[derive(Resource)]
pub struct CurrentNavMesh(pub Handle<NavMesh>);

#[derive(Component)]
struct Path {
    current: Vec3,
    next: Vec<Vec3>,
}

#[derive(Component, Clone)]
struct NavMeshDisp(Handle<NavMesh>);

fn setup(mut commands: Commands) {
    commands.spawn((
        NavMeshSettings { ..default() },
        NavMeshUpdateMode::Direct,
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));
}
fn view_navmesh(
    mut commands: Commands,
    navmeshes: Query<Entity, With<ManagedNavMesh>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyV) {
        if navmeshes.count() == 0 {
            info!("No managed navmeshes found!");
        } else {
            info!("Found navmesh!");
        }
        for entity in navmeshes {
            commands.entity(entity).insert(NavMeshDebug(RED_800.into()));
        }
    }
}
