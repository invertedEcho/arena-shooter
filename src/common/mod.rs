use bevy::prelude::*;

use crate::common::systems::handle_despawn_timer;

pub mod components;
pub mod systems;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_despawn_timer);
    }
}
