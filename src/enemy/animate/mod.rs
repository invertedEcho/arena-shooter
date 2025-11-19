use bevy::prelude::*;

use crate::enemy::animate::{
    messages::PlayEnemyAnimationMessage,
    systems::{
        link_enemy_animation, load_enemy_animations, play_enemy_animation,
        play_enemy_death_animation, setup_enemy_animation,
    },
};

mod components;
pub mod messages;
mod resources;
pub mod systems;

const TOTAL_ENEMY_MODEL_ANIMATIONS: usize = 24;

// TODO: give explicit name, maybe this needs to be done in Blender?
const ENEMY_MODEL_NAME: &str = "RootNode";

pub const ENEMY_MODEL_PATH: &str = "models/enemy/SWAT.glb";

pub struct AnimateEnemyPlugin;

impl Plugin for AnimateEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_enemy_animations)
            .add_systems(
                Update,
                (
                    setup_enemy_animation,
                    link_enemy_animation,
                    play_enemy_animation,
                    play_enemy_death_animation,
                ),
            )
            .add_message::<PlayEnemyAnimationMessage>();
    }
}

#[derive(Debug)]
pub enum EnemyAnimationType {
    Death,
    HitReceive,
    IdleGun,
    IdleGunPointing,
    Run,
}

fn get_animation_index_for_enemy_animation_type(
    enemy_animation_type: &EnemyAnimationType,
) -> usize {
    // https://poly.pizza/m/Btfn3G5Xv4 index is equal to list option select thing on preview
    match enemy_animation_type {
        EnemyAnimationType::Death => 0,
        EnemyAnimationType::HitReceive => 2,
        EnemyAnimationType::IdleGun => 5,
        EnemyAnimationType::IdleGunPointing => 6,
        EnemyAnimationType::Run => 16,
    }
}
