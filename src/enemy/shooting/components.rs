use bevy::prelude::*;

#[derive(Component)]
pub struct EnemyBullet;

#[derive(Component)]
pub struct EnemyShootPlayerCooldownTimer(pub Timer);
