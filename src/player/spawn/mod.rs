use avian3d::prelude::*;
use bevy::prelude::*;

use crate::player::{
    Player, PlayerMovementState, spawn::components::PlayerSpawnLocation,
};

pub mod components;

pub const PLAYER_CAPSULE_RADIUS: f32 = 0.1;
pub const PLAYER_CAPSULE_LENGTH: f32 = 0.8;

pub struct PlayerSpawnPlugin;

impl Plugin for PlayerSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerSpawnEvent>()
            .register_type::<PlayerSpawnLocation>()
            .add_systems(Update, handle_player_spawn_event);
    }
}

#[derive(Event)]
pub struct PlayerSpawnEvent {
    pub spawn_location: Vec3,
}

fn handle_player_spawn_event(
    mut commands: Commands,
    mut player_spawn_event_reader: EventReader<PlayerSpawnEvent>,
) {
    for event in player_spawn_event_reader.read() {
        commands.spawn((
            Player {
                health: 100.0,
                state: PlayerMovementState::Idle,
                on_ground: true,
            },
            Transform::from_translation(event.spawn_location),
            RigidBody::Kinematic,
            Collider::capsule(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_LENGTH),
            LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            LinearVelocity::ZERO,
            Visibility::Visible,
            CollisionEventsEnabled,
        ));
    }
}
