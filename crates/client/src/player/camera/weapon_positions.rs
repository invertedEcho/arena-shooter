use bevy::prelude::*;
use shared::{player::AimType, shooting::WeaponKind};

pub fn get_position_for_weapon(
    weapon_type: &WeaponKind,
    aim_type: &AimType,
) -> Vec3 {
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

    const NORMAL_POSITION_AK_47: Vec3 = Vec3 {
        x: 0.2,
        y: -0.25,
        z: -0.3,
    };
    const SCOPED_POSITION_ASSAULT_RIFLE: Vec3 = Vec3 {
        x: 0.0,
        y: -0.219,
        z: -0.3,
    };

    match weapon_type {
        WeaponKind::Glock => match aim_type {
            AimType::Normal => NORMAL_POSITION_PISTOL,
            AimType::Scoped => SCOPED_POSITION_PISTOL,
        },
        WeaponKind::AK47 | WeaponKind::P90 => match aim_type {
            AimType::Normal => NORMAL_POSITION_AK_47,
            AimType::Scoped => SCOPED_POSITION_ASSAULT_RIFLE,
        },
    }
}

pub fn get_muzzle_flash_position_for_weapon(
    weapon_type: &WeaponKind,
    aim_type: &AimType,
) -> Vec3 {
    const NORMAL_POSITION_PISTOL: Vec3 = Vec3::new(0.4, 0.05, 0.03);
    const NORMAL_POSITION_ASSAULT_RIFLE: Vec3 = Vec3::new(0.8, 0.07, 0.0);

    const SCOPED_POSITION_ASSAULT_RIFLE: Vec3 = Vec3::new(0.8, 0.1, 0.0);
    const SCOPED_POSITION_PISTOL: Vec3 = Vec3::new(0.5, 0.05, 0.0);

    match weapon_type {
        WeaponKind::Glock => match aim_type {
            AimType::Normal => NORMAL_POSITION_PISTOL,
            AimType::Scoped => SCOPED_POSITION_PISTOL,
        },
        WeaponKind::AK47 | WeaponKind::P90 => match aim_type {
            AimType::Normal => NORMAL_POSITION_ASSAULT_RIFLE,
            AimType::Scoped => SCOPED_POSITION_ASSAULT_RIFLE,
        },
    }
}
