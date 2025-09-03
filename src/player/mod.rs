use bevy::prelude::*;

use crate::player::{
    camera::PlayerCameraPlugin,
    components::Player,
    systems::{
        basic_shooting, handle_bullet_timer, player_movement, setup_player_camera,
        tick_player_weapon_timer,
    },
};

pub mod camera;
pub mod components;
mod systems;

const PLAYER_WALK_SPEED: f32 = 2.0;
const PLAYER_RUN_SPEED: f32 = 5.0;

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
                    handle_bullet_timer,
                    setup_player_camera,
                ),
            );
    }
}
