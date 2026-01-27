// pub fn check_collider_constructor_hierarchy_ready(
//     _trigger: On<ColliderConstructorHierarchyReady>,
//     colliders: Query<(&Collider, &GlobalTransform)>,
// ) {
//     let mut saved_colliders: Vec<(Collider, GlobalTransform)> = Vec::new();
//     for collider in colliders {
//         saved_colliders.push((collider.0.clone(), collider.1.clone()));
//     }
//     let json = serde_json::to_string(&saved_colliders);
//     match json {
//         Ok(serialized) => {
//             File::create("/home/invertedecho/Downloads/shooter-collider.json")
//                 .unwrap()
//                 .write_all(serialized.as_bytes());
//             info!("Saved collider file!");
//         }
//         Err(error) => {
//             error!("Failed to convert colliders to json string: {}", error);
//         }
//     }
// }
