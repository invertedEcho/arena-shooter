use bevy::prelude::*;

use crate::enemy::animate::systems::{
    handle_play_hit_animation_timer, link_enemy_animation,
    load_enemy_animations, play_enemy_hit_animation,
    reflect_enemy_state_to_current_animation, setup_enemy_animation,
};

mod components;
mod resources;
pub mod systems;

const TOTAL_ENEMY_MODEL_ANIMATIONS: usize = 24;
// https://poly.pizza/m/Btfn3G5Xv4 index is equal to list option select thing on preview
const ENEMY_DEATH_ANIMATION: usize = 0;
const _ENEMY_GUN_SHOOT_ANIMATION: usize = 1;
const ENEMY_HIT_RECEIVE_ANIMATION: usize = 2;
const ENEMY_IDLE_GUN_ANIMATION: usize = 5;
const ENEMY_IDLE_GUN_POINTING_ANIMATION: usize = 6;
const ENEMY_RUN_ANIMATION: usize = 16;

// TODO: give explicit name, maybe this needs to be done in Blender?
const ENEMY_MODEL_NAME: &str = "RootNode";

pub const ENEMY_MODEL_PATH: &str = "models/enemy/SWAT.glb";

pub struct AnimateEnemyPlugin;

impl Plugin for AnimateEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_enemy_animations).add_systems(
            Update,
            (
                setup_enemy_animation,
                reflect_enemy_state_to_current_animation,
                link_enemy_animation,
                play_enemy_hit_animation,
                handle_play_hit_animation_timer,
            ),
        );
    }
}
