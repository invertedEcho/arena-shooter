use bevy::prelude::*;

use crate::{
    player::{
        camera::{PlayerCameraPlugin, components::PlayerCameraState},
        hud::PlayerHudPlugin,
        shooting::{PlayerShootingPlugin, components::PlayerWeapons},
        spawn::PlayerSpawnPlugin,
    },
    shared::components::Health,
};

pub mod camera;
mod hud;
pub mod shooting;
pub mod spawn;

pub const DEFAULT_PLAYER_HEALTH: f32 = 100.0;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Player;

/// This component marks an entity as ready to be used for exterrnal systems that depend on the player, such as the HUD
#[derive(Component)]
pub struct PlayerReady;

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    health: Health,
    player_camera_state: PlayerCameraState,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            health: Health(DEFAULT_PLAYER_HEALTH),
            player_camera_state: PlayerCameraState::default(),
        }
    }
}

#[derive(Message)]
pub struct PlayerDeathMessage;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_systems(Update, mark_players_as_ready)
            .add_plugins(PlayerSpawnPlugin)
            .add_plugins(PlayerCameraPlugin)
            .add_plugins(PlayerShootingPlugin)
            .add_plugins(PlayerHudPlugin);
    }
}

type PlayersWithoutReadyMarker =
    (With<Player>, With<PlayerWeapons>, Without<PlayerReady>);

fn mark_players_as_ready(
    mut commands: Commands,
    query: Query<Entity, PlayersWithoutReadyMarker>,
) {
    for entity in query {
        commands.entity(entity).insert(PlayerReady);
    }
}
