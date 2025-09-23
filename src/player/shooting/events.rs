use bevy::prelude::*;

#[derive(Event)]
pub struct PlayerWeaponFiredEvent;

#[derive(Event)]
pub struct PlayerBulletHitEnemyEvent {
    pub enemy_hit: Entity,
}
