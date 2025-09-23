use crate::enemy::animate::play_enemy_hit_animation;
use crate::enemy::shooting::systems::{
    detect_player_bullet_collision_with_enemy, enemy_shoot_player,
    tick_enemy_shoot_player_cooldown_timer,
};
use bevy::prelude::*;

use crate::game_flow::GameState;

pub mod components;
mod systems;

pub struct EnemyShootingPlugin;

impl Plugin for EnemyShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                enemy_shoot_player,
                tick_enemy_shoot_player_cooldown_timer,
                // TODO: the detect_player_bullet_collision_with_enemy system may despawn an entity
                // that the play_enemy_hit_animation system will insert into. i think we could just
                // solve this issue with animating the enemy death, and at animation end then
                // despawn
                detect_player_bullet_collision_with_enemy
                    .after(play_enemy_hit_animation),
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}
