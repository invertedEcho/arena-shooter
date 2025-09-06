use bevy::prelude::*;

use crate::player::shooting::systems::{
    basic_shooting, detect_bullet_collision_with_player,
    tick_player_weapon_timer,
};

pub mod components;
mod systems;

#[derive(Resource)]
pub struct PlayerWeaponShootCooldownTimer(pub Timer);

pub struct PlayerShootingPlugin;

impl Plugin for PlayerShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                basic_shooting,
                tick_player_weapon_timer,
                detect_bullet_collision_with_player,
            ),
        );
    }
}
