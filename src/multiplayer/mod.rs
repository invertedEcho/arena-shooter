use bevy::prelude::*;
use lightyear::prelude::client::ClientPlugins;

mod protocol;

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientPlugins::default());
    }
}
