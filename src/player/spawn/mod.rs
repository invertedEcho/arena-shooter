use avian3d::prelude::*;
use bevy::prelude::*;

use crate::player::{Player, spawn::components::PlayerSpawnLocation};

pub mod components;

pub const PLAYER_CAPSULE_RADIUS: f32 = 0.2;
pub const PLAYER_CAPSULE_LENGTH: f32 = 1.3;

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
        let player_collider_shape =
            Collider::capsule(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_LENGTH);

        commands.spawn((
            Player::default(),
            Transform::from_translation(event.spawn_location),
            RigidBody::Kinematic,
            player_collider_shape,
            LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            LinearVelocity::ZERO,
            Visibility::Visible,
            CollisionEventsEnabled,
            CollidingEntities::default(),
        ));
    }
}
