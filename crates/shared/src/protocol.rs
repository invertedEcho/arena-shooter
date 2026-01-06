use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::player::Player;

pub struct PositionUpdateChannel;

// TODO: think about whether we need full transform, maybe just translation would be enough but
// think we need rotation too
#[derive(Serialize, Deserialize)]
pub struct ClientUpdatePositionMessage {
    pub new_translation: Vec3,
}

#[derive(Component, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ServerPosition {
    pub translation: Vec3,
}

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_channel::<PositionUpdateChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<ClientUpdatePositionMessage>()
            .add_direction(NetworkDirection::ClientToServer);

        app.register_component::<Player>();

        app.register_component::<ServerPosition>();
    }
}
