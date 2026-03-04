use bevy::prelude::*;
use shared::{
    player::{Player, PlayerReady},
    shooting::PlayerWeapons,
};

use crate::player::{
    camera::PlayerCameraPlugin, shooting::PlayerShootingPlugin,
};

pub mod camera;
pub mod shooting;

#[derive(Message)]
pub struct PlayerDeathMessage;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (mark_players_as_ready,))
            .add_plugins(PlayerCameraPlugin)
            .add_plugins(PlayerShootingPlugin);
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
        debug!("Marking player entity as ready");
        commands.entity(entity).insert(PlayerReady);
    }
}
