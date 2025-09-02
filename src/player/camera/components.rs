use bevy::prelude::*;

#[derive(Debug, Component, Reflect)]
#[reflect(Component, Default)]
pub struct PlayerCamera {
    pub mode: PlayerCameraMode,
    pub mouse_motion_enabled: bool,
}

#[derive(Debug, Default, PartialEq, Reflect)]
pub enum PlayerCameraMode {
    #[default]
    FirstPerson,
    ThirdPerson,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        PlayerCamera {
            mode: PlayerCameraMode::default(),
            mouse_motion_enabled: true,
        }
    }
}
