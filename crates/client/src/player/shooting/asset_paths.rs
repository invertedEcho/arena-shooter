use shared::shooting::WeaponKind;

pub const WEAPON_AK_47_MODEL_PATH: &str = "models/weapons/ak47/ak47.glb";
pub const WEAPON_GLOCK_MODEL_PATH: &str = "models/weapons/glock/glock.glb";

pub fn get_path_to_model_for_weapon_kind(weapon_type: &WeaponKind) -> String {
    match weapon_type {
        WeaponKind::Glock => WEAPON_GLOCK_MODEL_PATH.to_string(),
        WeaponKind::AK47 => WEAPON_AK_47_MODEL_PATH.to_string(),
        WeaponKind::P90 => "models/weapons/p90/p90.glb".to_string(),
    }
}
