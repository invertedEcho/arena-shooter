use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(PeerId);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub(crate) Color);

pub struct MultiplayerProtocolPlugin;

impl Plugin for MultiplayerProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<PlayerId>();
    }
}
