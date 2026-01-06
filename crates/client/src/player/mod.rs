use bevy::prelude::*;
use shared::player::{Player, PlayerReady};

use crate::player::{
    camera::PlayerCameraPlugin,
    hud::PlayerHudPlugin,
    shooting::{PlayerShootingPlugin, components::PlayerWeapons},
};

pub mod camera;
mod hud;
pub mod shooting;

#[derive(Message)]
pub struct PlayerDeathMessage;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (mark_players_as_ready,))
            .add_plugins(PlayerCameraPlugin)
            .add_plugins(PlayerShootingPlugin)
            .add_plugins(PlayerHudPlugin);
    }
}

type PlayersWithoutReadyMarker =
    (With<Player>, With<PlayerWeapons>, Without<PlayerReady>);

// hmm should this run on client?
fn mark_players_as_ready(
    mut commands: Commands,
    query: Query<Entity, PlayersWithoutReadyMarker>,
) {
    for entity in query {
        info!("Marking player entity as ready");
        commands.entity(entity).insert(PlayerReady);
    }
}
