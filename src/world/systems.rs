use bevy::prelude::*;

pub fn setup_world(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn((SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("main.gltf")),
    ),));
}
