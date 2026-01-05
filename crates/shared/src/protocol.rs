use avian3d::prelude::*;
use bevy::{ecs::entity::MapEntities, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::player::Player;

pub struct PositionUpdateChannel;

#[derive(Serialize, Deserialize)]
pub struct ClientUpdatePositionMessage {
    // client_id: ClientId,
    pub new_transform: Transform,
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

        app.register_component::<Position>();

        app.register_component::<Rotation>();
    }
}
