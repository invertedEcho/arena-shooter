use bevy::prelude::*;

use crate::enemy::{
    ai::EnemyAiPlugin, shooting::EnemyShootingPlugin, spawn::EnemySpawnPlugin,
};

pub mod ai;
pub mod shooting;
pub mod spawn;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnemySpawnPlugin)
            .add_plugins(EnemyAiPlugin)
            .add_plugins(EnemyShootingPlugin);
    }
}
