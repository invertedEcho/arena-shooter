use bevy::prelude::*;

use crate::player::{
    camera::PlayerCameraPlugin,
    movement::player_movement,
    shooting::{basic_shooting, billboad_muzzle_flash, tick_player_weapon_timer},
};

pub mod camera;
mod movement;
mod shooting;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_plugins(PlayerCameraPlugin)
            .add_systems(
                Update,
                (
                    player_movement,
                    basic_shooting,
                    tick_player_weapon_timer,
                    billboad_muzzle_flash,
                ),
            );
    }
}
