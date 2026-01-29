use bevy::prelude::*;

#[derive(Message)]
pub struct PlayerWeaponFiredMessage;

#[derive(Message)]
pub struct PlayerBulletHitEnemyMessage;

#[derive(Message)]
pub struct ReloadPlayerWeaponMessage;

#[derive(Message)]
pub struct PlayerWeaponSlotChangeMessage(pub usize);
