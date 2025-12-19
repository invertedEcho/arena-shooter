use bevy::prelude::*;

#[derive(Message)]
pub struct PlayerWeaponFiredMessage;

#[derive(Message)]
pub struct PlayerBulletHitEnemyMessage {
    pub enemy_hit: Entity,
    pub damage: f32,
}

#[derive(Message)]
pub struct ReloadPlayerWeaponMessage;

#[derive(Message)]
pub struct PlayerWeaponSlotChangeMessage(pub usize);
