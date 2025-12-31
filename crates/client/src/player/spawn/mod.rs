use avian3d::prelude::*;
use bevy::{color::palettes::css::WHITE, prelude::*};
use lightyear::prelude::{NetworkTarget, Replicate};

use crate::{
    game_flow::states::AppState,
    player::{
        Player, camera::messages::SpawnPlayerCamerasMessage,
        spawn::components::PlayerSpawnLocation,
    },
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
pub struct SpawnPlayerMessage {
    pub multiplayer: bool,
}

fn handle_player_spawn_event(
    mut commands: Commands,
    mut player_spawn_message_reader: MessageReader<SpawnPlayerMessage>,
    player_spawn_location: Single<&Transform, With<PlayerSpawnLocation>>,
    mut spawn_player_cameras_message_writer: MessageWriter<
        SpawnPlayerCamerasMessage,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_players: Query<Entity, With<Player>>,
) {
    for message in player_spawn_message_reader.read() {
        info!("read player spawn event, spawning player");

        spawn_player_cameras_message_writer.write(SpawnPlayerCamerasMessage);
    }
}
