use bevy::prelude::*;

#[derive(Component)]
pub struct MapDirectionalLight;

#[derive(Component)]
pub struct MapModel;

#[derive(Debug, Component)]
pub enum FloatDirection {
    Up,
    Down,
}
