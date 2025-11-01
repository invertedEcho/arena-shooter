use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    character_controller::components::CharacterControllerBundle,
    player::{Player, spawn::components::PlayerSpawnLocation},
};

pub mod components;

pub struct PlayerSpawnPlugin;

impl Plugin for PlayerSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnPlayerMessage>()
            .register_type::<PlayerSpawnLocation>()
            .add_systems(Update, handle_player_spawn_event);
    }
}

#[derive(Message)]
pub struct SpawnPlayerMessage;

fn handle_player_spawn_event(
    mut commands: Commands,
    mut player_spawn_message_reader: MessageReader<SpawnPlayerMessage>,
    player_spawn_location: Single<&Transform, With<PlayerSpawnLocation>>,
) {
    for _ in player_spawn_message_reader.read() {
        info!("read player spawn event, spawning player");

        commands.spawn((
            Name::new("Player"),
            Player::default(),
            Transform::from_translation(player_spawn_location.translation),
            Visibility::Visible,
            DebugRender::collider(Color::WHITE),
            CharacterControllerBundle::default(),
        ));
    }
}
