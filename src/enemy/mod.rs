use bevy::prelude::*;

use crate::enemy::{
    ai::{EnemyAiPlugin, components::EnemyState},
    animate::AnimateEnemyPlugin,
    shooting::EnemyShootingPlugin,
    spawn::{EnemySpawnLocation, EnemySpawnPlugin},
};

pub mod ai;
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
    pub state: EnemyState,
    pub health: f32,
}

impl Enemy {
    pub fn update_state(&mut self, new_state: EnemyState) {
        if self.state != new_state {
            println!("Enemy State change: {:?} -> {:?}", self.state, new_state);
            self.state = new_state;
        }
    }
}
