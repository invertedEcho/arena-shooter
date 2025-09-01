use bevy::prelude::*;

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct PlayerCamera {
    pub mode: PlayerCameraMode,
}

#[derive(Debug, Default, PartialEq, Reflect)]
pub enum PlayerCameraMode {
    #[default]
    FirstPerson,
    ThirdPerson,
}
