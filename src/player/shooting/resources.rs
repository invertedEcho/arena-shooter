use bevy::prelude::*;

#[derive(Resource)]
pub struct WeaponReloadTimer(pub Timer);

#[derive(Resource)]
pub struct ChangeWeaponCooldown(pub Timer);
