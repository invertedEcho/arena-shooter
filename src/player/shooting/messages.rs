use bevy::prelude::*;

#[derive(Message)]
pub struct PlayerWeaponFiredMessage;

#[derive(Message)]
pub struct PlayerBulletHitEnemyMessage {
    pub enemy_hit: Entity,
}
