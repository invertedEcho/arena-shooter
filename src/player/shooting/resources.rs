use bevy::prelude::*;

// FIXME: i think we only need timer if we just cancel reloading if weapon is switched
#[derive(Resource)]
pub struct WeaponReloadTimer {
    pub timer: Timer,
    /// The weapon slot that is being reloaded
    pub weapon_slot: usize,
}

#[derive(Resource)]
pub struct ChangeWeaponCooldown(pub Timer);
