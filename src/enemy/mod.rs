use bevy::prelude::*;

use crate::enemy::{
    ai::EnemyAiPlugin,
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
    pub state: EnemyState,
    pub health: f32,
}

#[derive(Default, Reflect, PartialEq, Debug)]
pub enum EnemyState {
    #[default]
    Idle,
    /// Check if the enemy can see the player
    CheckIfPlayerSeeable,
    /// Going to the location of the player
    ChasingPlayer,
    /// Enemy can see the player, will shoot the player now
    AttackPlayer,
    /// This state will be set when `enemy.health == 0.0`. A death animation will be played and
    /// afterwards the enemy will be despawned.
    Dead,
}
