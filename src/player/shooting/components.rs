use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerWeaponShootCooldownTimer(pub Timer);

#[derive(Component)]
pub struct MuzzleFlash;

#[derive(Component)]
pub struct PlayerWeapon;

#[derive(Component)]
pub struct Bullet;
