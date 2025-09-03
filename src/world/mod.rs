use bevy::prelude::*;

mod lighting;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world)
            .register_type::<Ground>();
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Ground;

fn setup_world(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn((SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("main.gltf")),
    ),));
}
