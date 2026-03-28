use shared::shooting::WeaponKind;

pub const ASSAULT_RIFLE_MODEL_PATH: &str = "weapons/assault_rifle.glb";
pub const PISTOL_MODEL_PATH: &str = "weapons/pistol.glb";

pub fn get_asset_path_for_weapon_type(weapon_type: &WeaponKind) -> String {
    match weapon_type {
        WeaponKind::Glock => PISTOL_MODEL_PATH.to_string(),
        WeaponKind::AK47 => ASSAULT_RIFLE_MODEL_PATH.to_string(),
    }
}
