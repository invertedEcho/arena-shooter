use bevy::prelude::*;
use std::fmt::Display;

use crate::shared::systems::disable_culling_for_skinned_meshes;

pub mod systems;

pub struct CommonPlugin;

#[derive(Reflect, PartialEq, Clone)]
pub enum WeaponType {
    Pistol,
    AssaultRifle,
}

impl Display for WeaponType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeaponType::Pistol => write!(f, "Pistol"),
            WeaponType::AssaultRifle => write!(f, "Assault Rifle"),
        }
    }
}

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, disable_culling_for_skinned_meshes);
    }
}

pub fn get_fire_delay_by_weapon_type(weapon_type: &WeaponType) -> f32 {
    match weapon_type {
        WeaponType::Pistol => 0.5,
        WeaponType::AssaultRifle => 0.2,
    }
}

/// Static information of the weapon
#[derive(Component, Clone)]
pub struct WeaponStats {
    /// The type of the weapon
    pub weapon_type: WeaponType,
    /// How much ammunition the weapon can hold at most (e.g. in the barrel)
    pub max_loaded_ammo: u32,
    pub weapon_slot_type: WeaponSlotType,
}

#[derive(Component)]
pub struct WeaponState {
    pub loaded_ammo: u32,
    pub carried_ammo: u32,
}

#[derive(PartialEq, Clone)]
pub enum WeaponSlotType {
    Primary,
    Secondary,
}
