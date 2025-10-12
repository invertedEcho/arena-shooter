use bevy::prelude::*;

use crate::{
    game_flow::states::InGameState,
    player::shooting::{
        events::{PlayerBulletHitEnemyEvent, PlayerWeaponFiredEvent},
        systems::{
            accurate_check_bullet_collision_for_impact_particle,
            detect_enemy_bullet_collision_with_player,
            handle_blood_screen_effect, handle_player_death_event,
            player_shooting, reload_player_weapon, setup_player_weapon,
            spawn_muzzle_flash, tick_player_weapon_shoot_cooldown_timer,
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
                    tick_player_weapon_shoot_cooldown_timer,
                    detect_enemy_bullet_collision_with_player,
                    handle_blood_screen_effect,
                    reload_player_weapon,
                    spawn_muzzle_flash,
                    accurate_check_bullet_collision_for_impact_particle,
                    setup_player_weapon,
                    handle_player_death_event,
                )
                    .run_if(in_state(InGameState::Playing)),
            );
    }
}
