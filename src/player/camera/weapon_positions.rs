use bevy::prelude::*;

use crate::shared::WeaponType;

const NORMAL_POSITION_PISTOL: Vec3 = Vec3 {
    x: 0.25,
    y: -0.25,
    z: -0.5,
};

const SCOPED_POSITION_PISTOL: Vec3 = Vec3 {
    x: 0.0,
    y: -0.2,
    z: -0.3,
};

const NORMAL_POSITION_ASSAULT_RIFLE: Vec3 = Vec3 {
    x: 0.2,
    y: -0.3,
    z: -0.5,
};
const SCOPED_POSITION_ASSAULT_RIFLE: Vec3 = Vec3 {
    x: 0.0,
    y: -0.3,
    z: -0.3,
};

#[derive(PartialEq, Clone)]
pub enum AimType {
    Normal,
    Scoped,
}

pub fn get_position_for_weapon(
    weapon_type: &WeaponType,
    aim_type: &AimType,
) -> Vec3 {
    match weapon_type {
        WeaponType::Pistol => match aim_type {
            AimType::Normal => NORMAL_POSITION_PISTOL,
            AimType::Scoped => SCOPED_POSITION_PISTOL,
        },
        WeaponType::AssaultRifle => match aim_type {
            AimType::Normal => NORMAL_POSITION_ASSAULT_RIFLE,
            AimType::Scoped => SCOPED_POSITION_ASSAULT_RIFLE,
        },
    }
}

// TODO: this must change depending on the cameras FOV
pub fn get_muzzle_flash_position_for_weapon(
    weapon_type: &WeaponType,
    aim_type: &AimType,
) -> Vec3 {
    const NORMAL_POSITION_PISTOL: Vec3 = Vec3::new(0.4, 0.05, 0.03);
    const NORMAL_POSITION_ASSAULT_RIFLE: Vec3 = Vec3::new(0.8, 0.07, 0.0);
    match weapon_type {
        WeaponType::Pistol => match aim_type {
            AimType::Normal => NORMAL_POSITION_PISTOL,
            AimType::Scoped => Vec3::ZERO,
        },
        WeaponType::AssaultRifle => match aim_type {
            AimType::Normal => NORMAL_POSITION_ASSAULT_RIFLE,
            AimType::Scoped => Vec3::ZERO,
        },
    }
}
