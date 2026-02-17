use bevy::prelude::*;

#[derive(Message)]
pub struct PlayerWeaponFiredMessage;

#[derive(Message)]
pub struct PlayerBulletHitEnemyMessage;

#[derive(Message)]
pub struct ReloadPlayerWeaponMessage;

/// This message gets sent when the player changes his active Weapon.
/// 0: The new weapon slot index
#[derive(Message)]
pub struct PlayerWeaponSlotChangeMessage(pub usize);
