use std::sync::Arc;

use bevy::prelude::*;
use bevy_landmass::{NavMesh3d, nav_mesh::bevy_mesh_to_landmass_nav_mesh};

// A utility to wait for a mesh to be loaded and convert the mesh to a nav mesh.
#[derive(Component)]
pub struct ConvertMesh {
    pub mesh: Handle<Mesh>,
    pub nav_mesh: Handle<NavMesh3d>,
}

pub fn convert_mesh(
    converters: Query<(Entity, &ConvertMesh)>,
    meshes: Res<Assets<Mesh>>,
    mut nav_meshes: ResMut<Assets<NavMesh3d>>,
    mut commands: Commands,
) {
    for (entity, converter) in converters.iter() {
        let Some(mesh) = meshes.get(&converter.mesh) else {
            continue;
        };

        let nav_mesh = bevy_mesh_to_landmass_nav_mesh(mesh).unwrap();

        match nav_mesh.validate() {
            Ok(valid_nav_mesh) => {
                nav_meshes
                    .insert(
                        &converter.nav_mesh,
                        NavMesh3d {
                            nav_mesh: Arc::new(valid_nav_mesh),
                        },
                    )
                    .unwrap();
                commands.entity(entity).remove::<ConvertMesh>();
            }
            Err(error) => {
                error!("Failed to validate nav mesh: {}", error);
            }
        }
    }
}
