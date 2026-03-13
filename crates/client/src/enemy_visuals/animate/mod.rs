use bevy::prelude::*;
use shared::enemy::components::EnemyState;

use crate::enemy_visuals::animate::{
    messages::PlayEnemyAnimationMessage,
    systems::{
        link_enemy_animation, load_enemy_animations,
        play_animation_on_changed_health, play_enemy_animation,
        setup_enemy_animation, update_animation_on_enemy_state_change,
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
                    update_animation_on_enemy_state_change,
                    play_animation_on_changed_health,
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

fn get_animation_type_for_enemy_state(
    enemy_state: &EnemyState,
) -> EnemyAnimationType {
    match enemy_state {
        EnemyState::Idle => EnemyAnimationType::IdleGun,
        EnemyState::PlayerInFOV => EnemyAnimationType::IdleGun,
        EnemyState::Dead => EnemyAnimationType::Death,
        EnemyState::GoToAgentTarget => EnemyAnimationType::Run,
        EnemyState::EnemyAgentReachedTarget => EnemyAnimationType::IdleGun,
        EnemyState::AttackPlayer(_) => EnemyAnimationType::IdleGunPointing,
        EnemyState::RotateTowardsPlayer => EnemyAnimationType::IdleGun,
    }
}
