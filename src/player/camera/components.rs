use bevy::prelude::*;

#[derive(Debug, Component, Default)]
pub struct PlayerCamera;

#[derive(Debug, Default, PartialEq)]
pub enum PlayerCameraState {
    #[default]
    Normal,
    FreeCam,
}

#[derive(Component)]
pub struct FreeCam;
