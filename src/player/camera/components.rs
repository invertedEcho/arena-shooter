use bevy::prelude::*;

#[derive(Debug, Component, Default)]
pub struct PlayerCamera {
    pub state: PlayerCameraState,
}

#[derive(Debug, Default, PartialEq)]
pub enum PlayerCameraState {
    #[default]
    Normal,
    FreeCam,
}

#[derive(Component)]
pub struct FreeCam;
