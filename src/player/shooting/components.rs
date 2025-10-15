use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerShootCooldownTimer(pub Timer);

#[derive(Component)]
pub struct MuzzleFlash;

#[derive(Component)]
pub struct PlayerWeapon {
    pub loaded_ammo: u32,
    pub carried_ammo: u32,
    pub max_loaded_ammo: u32,
    pub reloading: bool,
}

#[derive(Component)]
pub struct PlayerBullet {
    pub damage: f32,
}

#[derive(Component)]
pub struct BloodScreenEffect {
    pub timer: Timer,
    pub total_timer_iteration_count: f32,
    pub currrent_timer_iteration: u32,
}

const DEFAULT_BLOOD_SCREEN_TIMER_DURATION: f32 = 0.1;
/// The default for BloodScreenEffect, e.g. used in conjunction with Blood Screen png, starting
/// with color tint alpha set to 1.0, and every 0.1 second decreasing until alpha has reached 0.0
impl Default for BloodScreenEffect {
    fn default() -> Self {
        BloodScreenEffect {
            timer: Timer::from_seconds(
                DEFAULT_BLOOD_SCREEN_TIMER_DURATION,
                TimerMode::Repeating,
            ),
            total_timer_iteration_count: 1.0
                / DEFAULT_BLOOD_SCREEN_TIMER_DURATION,
            // lol no zero index in what world do you live in?
            currrent_timer_iteration: 1,
        }
    }
}
