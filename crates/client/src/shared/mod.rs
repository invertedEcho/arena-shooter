use bevy::prelude::*;
use shared::shooting::WeaponType;

use crate::{
    game_flow::states::InGameState,
    shared::systems::{
        disable_culling_for_skinned_meshes, hide_only_visible_in_game,
        show_only_visible_in_game,
    },
};

pub mod components;
pub mod systems;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, disable_culling_for_skinned_meshes);
        app.add_systems(
            OnEnter(InGameState::Playing),
            show_only_visible_in_game,
        );
        app.add_systems(
            OnExit(InGameState::Playing),
            hide_only_visible_in_game,
        );
    }
}

pub fn get_fire_delay_by_weapon_type(weapon_type: &WeaponType) -> f32 {
    match weapon_type {
        WeaponType::Pistol => 0.3,
        WeaponType::AssaultRifle => 0.115,
    }
}
