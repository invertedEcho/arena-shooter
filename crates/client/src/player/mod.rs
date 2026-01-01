use bevy::prelude::*;
use shared::player::{Player, PlayerReady};

use crate::player::{
    hud::PlayerHudPlugin,
    shooting::{PlayerShootingPlugin, components::PlayerWeapons},
};

mod camera;
mod hud;
pub mod shooting;

#[derive(Message)]
pub struct PlayerDeathMessage;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mark_players_as_ready)
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
