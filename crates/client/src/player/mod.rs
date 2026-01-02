use bevy::prelude::*;
use lightyear::input::client::InputSystems;
use shared::player::{Player, PlayerReady};

use crate::player::{
    camera::PlayerCameraPlugin,
    hud::PlayerHudPlugin,
    input::{buffer_input, handle_client_input},
    shooting::{PlayerShootingPlugin, components::PlayerWeapons},
};

pub mod camera;
mod hud;
pub mod input;
pub mod shooting;

#[derive(Message)]
pub struct PlayerDeathMessage;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            buffer_input.in_set(InputSystems::WriteClientInputs),
        )
        .add_systems(FixedUpdate, handle_client_input)
        .add_systems(Update, (mark_players_as_ready,))
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
