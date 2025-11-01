use bevy::prelude::*;

#[derive(Component)]
pub struct EnemyBullet;

/// A timer so that enemies don't just shoot every frame but every x seconds
#[derive(Component)]
pub struct EnemyShootCooldownTimer(pub Timer);
