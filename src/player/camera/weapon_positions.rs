use bevy::prelude::*;

use crate::shared::WeaponType;

const NORMAL_POSITION_ASSAULT_RIFLE: Vec3 = Vec3 {
    x: 0.4,
    y: -0.2,
    z: -1.0,
};
const SCOPED_POSITION_ASSAULT_RIFLE: Vec3 = Vec3 {
    x: 0.0,
    y: -0.05,
    z: -0.3,
};

#[derive(PartialEq)]
pub enum AimType {
    Normal,
    Scoped,
}

pub fn get_position_for_weapon(
    weapon_type: &WeaponType,
    aim_type: AimType,
) -> Vec3 {
    match weapon_type {
        WeaponType::AssaultRifle => match aim_type {
            AimType::Normal => NORMAL_POSITION_ASSAULT_RIFLE,
            AimType::Scoped => SCOPED_POSITION_ASSAULT_RIFLE,
        },
        WeaponType::Pistol => match aim_type {
            AimType::Normal => NORMAL_POSITION_ASSAULT_RIFLE,
            AimType::Scoped => SCOPED_POSITION_ASSAULT_RIFLE,
        },
    }
}
