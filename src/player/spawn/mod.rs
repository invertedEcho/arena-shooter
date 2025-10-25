use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    kinematic_controller::CharacterControllerBundle,
    player::{Player, spawn::components::PlayerSpawnLocation},
};

pub mod components;

pub const PLAYER_CAPSULE_RADIUS: f32 = 0.2;
pub const PLAYER_CAPSULE_LENGTH: f32 = 1.4;

pub struct PlayerSpawnPlugin;

impl Plugin for PlayerSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayerSpawnMessage>()
            .register_type::<PlayerSpawnLocation>()
            .add_systems(Update, handle_player_spawn_event);
    }
}

#[derive(Message)]
pub struct PlayerSpawnMessage {
    pub spawn_location: Vec3,
}

fn handle_player_spawn_event(
    mut commands: Commands,
    mut player_spawn_message_reader: MessageReader<PlayerSpawnMessage>,
) {
    for event in player_spawn_message_reader.read() {
        info!("read player spawn event, spawning player");

        commands.spawn((
            Name::new("Player"),
            Player::default(),
            Transform::from_translation(event.spawn_location),
            Visibility::Visible,
            DebugRender::collider(Color::WHITE),
            CharacterControllerBundle::default(),
        ));
    }
}
