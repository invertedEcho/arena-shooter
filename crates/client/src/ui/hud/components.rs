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
pub struct CurrentWaveText;

#[derive(Component)]
pub struct EnemiesLeftText;

#[derive(Component)]
pub struct PlayerCrosshair;

#[derive(Component)]
pub struct PlayerWeaponText(pub usize);

/// Each DamageIndicator has a Timer, so we can decrease alpha of the image each X seconds
#[derive(Component)]
pub struct DamageIndicator(pub Timer);

#[derive(Component)]
pub struct CurrentCashAmount;

#[derive(Component)]
pub struct CurrentWaveFinishedText;

#[derive(Component)]
pub struct NextWaveTimerText;
