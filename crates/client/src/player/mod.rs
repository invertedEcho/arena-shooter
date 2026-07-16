use bevy::prelude::*;
use netvy::Owned;
use shared::{
    player::{OurPlayerReady, Player},
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
        app.add_systems(FixedUpdate, mark_players_as_ready)
            .add_plugins(PlayerCameraPlugin)
            .add_plugins(PlayerShootingPlugin);
    }
}

type PlayersWithoutReadyMarker = (
    With<Player>,
    With<PlayerWeapons>,
    // we only insert PlayerReady component into our own player.
    With<Owned>,
    Without<OurPlayerReady>,
);

// hmm should this run on client?
fn mark_players_as_ready(
    mut commands: Commands,
    query: Query<Entity, PlayersWithoutReadyMarker>,
) {
    for entity in query {
        debug!("Marking player {entity} as ready");
        commands.entity(entity).insert(OurPlayerReady);
    }
}
