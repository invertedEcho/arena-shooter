use bevy::prelude::*;
use lightyear::prelude::AppComponentExt;
use serde::{Deserialize, Serialize};

use crate::player::Player;

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct PlayerPosition(pub Vec3);

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct PlayerId(pub u32);

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Player>();
    }
}
