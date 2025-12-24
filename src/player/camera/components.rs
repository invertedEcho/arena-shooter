use bevy::prelude::*;

/// This camera is for rendering the whole world
/// It has RenderLayer 0
#[derive(Component)]
pub struct WorldCamera;

/// This camera is for rendering everything that should be above the world, so models dont clip
/// into walls for example. Right now it only holds the PlayerWeaponModel.
/// It has RenderLayer 1 so its rendered on top of WorldCamera
#[derive(Debug, Component, Default)]
pub struct ViewModelCamera;

#[derive(Component, Debug, Default, PartialEq, Reflect)]
pub enum PlayerCameraState {
    #[default]
    Normal,
    FreeCam,
}

#[derive(Component)]
pub struct PlayerWeaponModel;

#[derive(Component)]
pub struct FreeCam;

#[derive(Component)]
pub struct MuzzleFlash;

#[derive(Component)]
pub struct InterpolateWeapon {
    pub target_position: Vec3,
}
