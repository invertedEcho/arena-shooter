use crate::shared::WeaponType;

pub const ASSAULT_RIFLE_MODEL_PATH: &str = "weapons/assault_rifle.glb";
pub const PISTOL_MODEL_PATH: &str = "weapons/pistol.glb";

pub fn get_asset_path_for_weapon_type(weapon_type: &WeaponType) -> String {
    match weapon_type {
        WeaponType::Pistol => PISTOL_MODEL_PATH.to_string(),
        WeaponType::AssaultRifle => ASSAULT_RIFLE_MODEL_PATH.to_string(),
    }
}
