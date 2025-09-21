use bevy::prelude::*;

use crate::{
    common::BULLET_VELOCITY,
    enemy::{
        ai::{EnemyAiPlugin, EnemyAiState},
        animate::AnimateEnemyPlugin,
        shooting::EnemyShootingPlugin,
        spawn::{EnemySpawnLocation, EnemySpawnPlugin},
    },
};

mod ai;
mod animate;
pub mod shooting;
mod spawn;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnemySpawnPlugin)
            .add_plugins(AnimateEnemyPlugin)
            .add_plugins(EnemyAiPlugin)
            .add_plugins(EnemyShootingPlugin)
            .register_type::<Enemy>()
            .register_type::<EnemySpawnLocation>();
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Enemy {
    state: EnemyAiState,
    pub health: f32,
}
