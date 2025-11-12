use bevy::prelude::*;

// TODO: Document why we store this Handle
#[derive(Resource)]
pub struct WorldSceneHandle(pub Handle<Scene>);
