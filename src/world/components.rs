use bevy::prelude::*;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Ground;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Wall;

#[derive(Component)]
pub struct DebugPoint;
