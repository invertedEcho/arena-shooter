use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerShootCooldownTimer(pub Timer);

/// original_rotation gets updated whenever the player uses the mouse to look around.
/// This way, we can have a camera kickback effect when shooting, but also know, to what value we
/// need to slerp back to
#[derive(Component, Default)]
pub struct ShootRecoil {
    pub original_rotation: Quat,
}
