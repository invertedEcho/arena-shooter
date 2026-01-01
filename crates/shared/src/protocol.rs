use bevy::prelude::*;
use lightyear::prelude::AppComponentExt;

use crate::player::Player;

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // lightyear protocol
        app.register_component::<Player>();
    }
}
