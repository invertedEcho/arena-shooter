use bevy::{
    asset::weak_handle,
    color::palettes::{self, css::RED},
    gltf::GltfMesh,
    prelude::*,
};
use vleue_navigator::prelude::*;

use crate::{enemy::Enemy, player::Player};

pub struct NavMeshPathfindingPlugin;

impl Plugin for NavMeshPathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            VleueNavigatorPlugin,
            NavmeshUpdaterPlugin::<PrimitiveObstacle>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                setup_scene,
                get_path_from_enemy_to_player_on_enter,
                change_navmesh_visibility,
            ),
        );
    }
}

const HANDLE_TRIMESH_OPTIMIZED: Handle<NavMesh> =
    weak_handle!("100AD183-2C5C-49A1-AB32-142000E87828");

#[derive(Resource, Default, Deref)]
struct GltfHandle(Handle<Gltf>);

#[derive(Resource)]
pub struct CurrentNavMesh(pub Handle<NavMesh>);

#[derive(Component, Clone)]
pub struct NavMeshDisp(Handle<NavMesh>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GltfHandle(
        asset_server.load("maps/main/navmesh.gltf"),
    ));
    commands.insert_resource(CurrentNavMesh(HANDLE_TRIMESH_OPTIMIZED));
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    gltfs: Res<Assets<Gltf>>,
    gltf: Res<GltfHandle>,
    mut navmeshes: ResMut<Assets<NavMesh>>,
    mut is_done: Local<bool>,
) {
    // TODO: properly implement loading and setup
    if *is_done {
        return;
    }
    if let Some(gltf) = gltfs.get(gltf.id()) {
        let navmesh = NavMesh::from_bevy_mesh(
            meshes
                .get(
                    &gltf_meshes
                        .get(&gltf.named_meshes["navmesh"])
                        .unwrap()
                        .primitives[0]
                        .mesh,
                )
                .unwrap(),
        )
        .unwrap();

        let mut material: StandardMaterial =
            Color::Srgba(palettes::css::ANTIQUE_WHITE).into();
        material.unlit = true;
        commands.spawn((
            Mesh3d(meshes.add(navmesh.to_wireframe_mesh())),
            MeshMaterial3d(materials.add(material)),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::Hidden,
            NavMeshDisp(HANDLE_TRIMESH_OPTIMIZED),
        ));

        navmeshes.insert(&HANDLE_TRIMESH_OPTIMIZED, navmesh);

        *is_done = true;
    }
}

fn get_path_from_enemy_to_player_on_enter(
    mut commands: Commands,
    enemy_transform_query: Query<&Transform, With<Enemy>>,
    navmeshes: Res<Assets<NavMesh>>,
    current_mesh: Res<CurrentNavMesh>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_transform: Single<&Transform, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        let navmesh = navmeshes.get(&current_mesh.0).unwrap();
        for enemy_transform in enemy_transform_query {
            let path = navmesh.transformed_path(
                Vec3 {
                    x: enemy_transform.translation.x,
                    y: 0.0,
                    z: enemy_transform.translation.z,
                },
                Vec3 {
                    x: player_transform.translation.x,
                    y: 0.0,
                    z: player_transform.translation.z,
                },
            );
            match path {
                Some(res) => {
                    for coord in res.path {
                        commands.spawn((
                            Mesh3d(meshes.add(Sphere::new(0.1))),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: RED.into(),
                                ..Default::default()
                            })),
                            Transform::from_translation(coord),
                        ));
                    }
                }
                None => {
                    warn!("Could not find path from enemy to player");
                }
            }
        }
    }
}

fn change_navmesh_visibility(
    mut query: Query<(&mut Visibility, &NavMeshDisp)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_mesh: Res<CurrentNavMesh>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyV) {
        if query.iter().len() == 0 {
            error!("Could not find any navmeshdisp to change Visibility!");
        }
        for (mut visible, nav) in query.iter_mut() {
            if nav.0 == current_mesh.0 {
                match *visible {
                    Visibility::Visible => *visible = Visibility::Hidden,
                    Visibility::Hidden => *visible = Visibility::Visible,
                    Visibility::Inherited => *visible = Visibility::Inherited,
                }
            }
        }
    }
}
