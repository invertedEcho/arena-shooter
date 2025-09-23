use bevy::prelude::*;

use crate::enemy::animate::{
    events::PlayEnemyDeathAnimationEvent,
    systems::{
        handle_play_enemy_death_animation_event, handle_play_hit_animation,
        link_enemy_animation, load_enemy_animations, play_enemy_hit_animation,
        setup_enemy_animation, update_enemy_animation_on_state_changed,
    },
};

mod components;
pub mod events;
mod resources;
pub mod systems;

const TOTAL_ENEMY_MODEL_ANIMATIONS: usize = 24;
// https://poly.pizza/m/Btfn3G5Xv4 index is equal to list option select thing on preview
const ENEMY_DEATH_ANIMATION: usize = 0;
const _ENEMY_GUN_SHOOT_ANIMATION: usize = 1;
const ENEMY_HIT_RECEIVE_ANIMATION: usize = 2;
const _ENEMY_IDLE_ANIMATION: usize = 4;
const ENEMY_IDLE_GUN_ANIMATION: usize = 5;
const ENEMY_IDLE_GUN_POINTING_ANIMATION: usize = 6;

pub const SWAT_MODEL_PATH: &str = "models/animated/SWAT.glb";

pub struct AnimateEnemyPlugin;

impl Plugin for AnimateEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayEnemyDeathAnimationEvent>()
            .add_systems(Startup, load_enemy_animations)
            .add_systems(
                Update,
                (
                    setup_enemy_animation,
                    update_enemy_animation_on_state_changed,
                    link_enemy_animation,
                    play_enemy_hit_animation,
                    handle_play_hit_animation,
                    handle_play_enemy_death_animation_event,
                ),
            );
    }
}
