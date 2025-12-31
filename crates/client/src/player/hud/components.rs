use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerHud;

#[derive(Component)]
pub struct PlayerHealthText;

#[derive(Component)]
pub struct PlayerLoadedAmmoText;

#[derive(Component)]
pub struct PlayerCarriedAmmoText;

#[derive(Component)]
pub struct PlayerScoreText;

#[derive(Component)]
pub struct EnemyScoreText;

#[derive(Component)]
pub struct CurrentWaveText;

#[derive(Component)]
pub struct EnemiesLeftText;

#[derive(Component)]
pub struct PlayerCrosshair;

#[derive(Component)]
pub struct PlayerWeaponText(pub usize);
