use bevy::prelude::*;

use crate::{
    game_flow::AppState,
    player::shooting::{
        events::{PlayerBulletHitEnemyEvent, PlayerWeaponFiredEvent},
        systems::{
            accurate_check_bullet_collision_for_impact_particle,
            detect_enemy_bullet_collision_with_player,
            handle_blood_screen_effect, player_shooting, reload_player_weapon,
            spawn_muzzle_flash, tick_player_weapon_timer,
        },
    },
};

pub mod components;
pub mod events;
mod systems;

pub struct PlayerShootingPlugin;

impl Plugin for PlayerShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerWeaponFiredEvent>()
            .add_event::<PlayerBulletHitEnemyEvent>()
            .add_systems(
                Update,
                (
                    player_shooting,
                    tick_player_weapon_timer,
                    detect_enemy_bullet_collision_with_player,
                    handle_blood_screen_effect,
                    reload_player_weapon,
                    spawn_muzzle_flash,
                    accurate_check_bullet_collision_for_impact_particle,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
