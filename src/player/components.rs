use bevy::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerWeaponShootCooldownTimer(pub Timer);

#[derive(Component)]
pub struct DespawnTimer(pub Timer);
