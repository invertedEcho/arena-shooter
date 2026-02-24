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
/// hit_point: Where the bullet hit exactly
/// entity_hit: Which entity got hit by the bullet
#[derive(Message)]
pub struct PlayerBulletHit {
    pub hit_point: Vec3,
    pub entity_hit: Entity,
}
