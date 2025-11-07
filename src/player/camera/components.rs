use bevy::prelude::*;

#[derive(Debug, Component, Default)]
pub struct ViewModelCamera;

#[derive(Component)]
pub struct WorldModelCamera;

#[derive(Debug, Default, PartialEq, Reflect)]
pub enum PlayerCameraState {
    #[default]
    Normal,
    FreeCam,
}

#[derive(Component)]
pub struct PlayerWeaponModel;

#[derive(Component)]
pub struct FreeCam;
