use bevy::prelude::*;

use crate::enemy::{
    ai::{EnemyAiPlugin, EnemyState},
    animate::AnimateEnemyPlugin,
    shooting::EnemyShootingPlugin,
    spawn::{EnemySpawnLocation, EnemySpawnPlugin},
};

mod ai;
mod animate;
pub mod shooting;
pub mod spawn;

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
    state: EnemyState,
    pub health: f32,
    // TODO: doesnt belong here, should be in EnemyAI plugin, but i first need to figure out how i
    // actually wanna do this, so for now it will just live here.
    // enemy
    // I THINK I GOT IT: insert path component into enemy.
    current_patrol_destination: Option<Vec3>,
    next_patrol_destinations: Option<Vec<Vec3>>,
}
