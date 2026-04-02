use bevy::prelude::*;

/// This message can be written to trigger a player weapon model "refresh", such as when
/// buying a new weapon or changing the currently active player weapon
#[derive(Message)]
pub struct UpdatePlayerWeaponModel;
