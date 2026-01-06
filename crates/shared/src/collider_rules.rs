use crate::SelectedMapState;
use avian3d::prelude::ColliderConstructor;

// TODO: validate the collider rules, e.g. whether these Names actually exist and have a mesh
// defined
// NOTE:
// To get the correct name to use here when inspecting entities in Blender:
// Must be the actual Mesh, e.g. the green triangle, followed by a dot, and then the
// material name. Case-sensitive.
// Alternatively, use this query: Query<(Entity, &Name), With<Mesh3d>>,
// This will give you the exact name to use
pub fn get_collider_rules_by_map(
    selected_map: &SelectedMapState,
) -> Vec<(&'static str, std::option::Option<ColliderConstructor>)> {
    match selected_map {
        SelectedMapState::MediumPlastic => {
            vec![
                (
                    "OuterWallsMesh.material.002",
                    Some(ColliderConstructor::TrimeshFromMesh),
                ),
                (
                    "InnerHouseMesh.material_4",
                    Some(ColliderConstructor::TrimeshFromMesh),
                ),
                (
                    "OuterHouseMesh.material.001",
                    Some(ColliderConstructor::TrimeshFromMesh),
                ),
                (
                    "Cube.016_0.material_11",
                    Some(ColliderConstructor::TrimeshFromMesh),
                ),
                (
                    "Cube.004_0.material_11",
                    Some(ColliderConstructor::TrimeshFromMesh),
                ),
                (
                    "up2_0.material_11",
                    Some(ColliderConstructor::TrimeshFromMesh),
                ),
                (
                    "Cube.005_0.material.002",
                    Some(ColliderConstructor::TrimeshFromMesh),
                ),
                (
                    "Cube.005_1.material_4",
                    Some(ColliderConstructor::TrimeshFromMesh),
                ),
            ]
        }
        SelectedMapState::TinyTown => {
            vec![]
        }
    }
}
