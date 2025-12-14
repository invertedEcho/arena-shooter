use bevy::prelude::*;

use crate::shared::systems::{
    disable_culling_for_skinned_meshes, handle_despawn_timer,
};

pub mod components;
pub mod systems;

pub const DEFAULT_BULLET_DAMAGE: f32 = 7.5;

pub struct CommonPlugin;

#[derive(Reflect)]
pub enum WeaponType {
    AssaultRifle,
    Pistol,
}

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_despawn_timer, disable_culling_for_skinned_meshes),
        );
    }
}

pub fn get_fire_delay_by_weapon_type(weapon_type: &WeaponType) -> f32 {
    match weapon_type {
        WeaponType::AssaultRifle => 0.2,
        WeaponType::Pistol => 0.5,
    }
}
