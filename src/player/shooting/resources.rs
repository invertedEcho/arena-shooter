use bevy::prelude::*;

#[derive(Resource)]
pub struct PlayerWeaponReloadTimer {
    pub timer: Timer,
    /// The weapon slot that is being reloaded
    pub weapon_slot: usize,
}
