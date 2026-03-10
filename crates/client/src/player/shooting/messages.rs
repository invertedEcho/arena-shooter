use bevy::prelude::*;

#[derive(Message)]
pub struct PlayerWeaponFiredMessage;

#[derive(Message)]
pub struct ReloadPlayerWeaponMessage;

/// This message gets sent when the player changes his active Weapon.
/// 0: The new weapon slot index
#[derive(Message)]
pub struct PlayerWeaponSlotChangeMessage(pub usize);

/// This message gets written whenever a player shoots, and the bullet hits something
#[derive(Message)]
pub struct PlayerBulletHit {
    /// Where the bullet hit exactly
    pub hit_point: Vec3,
    /// Which entity got hit by the bullet
    pub entity_hit: Entity,
    pub hit_normal: Vec3,
}
