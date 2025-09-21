use bevy::prelude::*;

#[derive(Event)]
pub struct PlayerWeaponFiredEvent;

#[derive(Event)]
pub struct PlayerBulletHitEnemy {
    pub enemy_hit: Entity,
}
