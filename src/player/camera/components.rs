use bevy::prelude::*;

#[derive(Debug, Component, Default)]
pub struct ViewModelCamera;

// #[derive(Debug, Default, PartialEq)]
// pub enum PlayerCameraState {
//     #[default]
//     Normal,
//     // FreeCam,
// }

#[derive(Component)]
pub struct PlayerWeaponModel;

#[derive(Component)]
pub struct FreeCam;
