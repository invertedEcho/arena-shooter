use bevy::prelude::*;

// TODO: if we have more than two of these, merge them to one, WorldOBjectSpawnLocation and just
// give them a type enum
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MedkitSpawnLocation;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AmmunitionSpawnLocation;

#[derive(Component)]
pub struct RespawnTimer(pub Timer);
