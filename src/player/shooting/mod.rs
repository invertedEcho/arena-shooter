use bevy::prelude::*;

use crate::player::shooting::systems::{
    basic_shooting, billboad_muzzle_flash, tick_player_weapon_timer,
};

pub mod components;
mod systems;

pub struct PlayerShootingPlugin;

impl Plugin for PlayerShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                basic_shooting,
                billboad_muzzle_flash,
                tick_player_weapon_timer,
            ),
        );
    }
}
