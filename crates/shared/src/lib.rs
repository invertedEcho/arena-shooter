use avian3d::{PhysicsPlugins, prelude::*};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{components::DespawnTimer, protocol::ProtocolPlugin};

pub mod character_controller;
pub mod components;
pub mod enemy;
pub mod game_score;
pub mod multiplayer_messages;
pub mod player;
pub mod protocol;
pub mod shooting;
pub mod utils;
pub mod world_object;

pub const DEFAULT_HEALTH: f32 = 100.0;

#[derive(Resource, PartialEq)]
pub enum ServerRunMode {
    Headless,
    Headful,
}

/// A state indicating whether we are using a remote server to connect to,
/// or a local server is started, e.g. for singleplayer
#[derive(States, PartialEq, Debug, Hash, Clone, Eq)]
pub enum ServerMode {
    RemoteServer,
    LocalServerSinglePlayer,
}

/// The game mode that is running on the server. Must be a component as we can only replicate
/// components with lightyear.
#[derive(Component, Serialize, Deserialize, PartialEq, Debug)]
pub enum GameModeServer {
    Waves,
    FreeForAll,
    FreeRoam,
}

#[derive(
    States, Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize,
)]
pub enum GameStateServer {
    #[default]
    Running,
    Paused,
}

#[derive(Message)]
pub struct SpawnDebugSphereMessage {
    pub location: Vec3,
    pub color: Color,
    pub radius: f32,
}

impl SpawnDebugSphereMessage {
    pub fn _new<T: Into<Vec3>, U: Into<Color>>(
        point: T,
        color: U,
        radius: f32,
    ) -> Self {
        Self {
            location: point.into(),
            color: color.into(),
            radius,
        }
    }
}

/// 0: The enemy entity that was killed
#[derive(Message)]
pub struct EnemyKilledMessage(pub Entity);

pub const GRAVITY: f32 = 9.81;

pub const TINY_TOWN_MAP_PATH: &str = "maps/tiny_town/main.gltf";
pub const MEDIUM_PLASTIC_MAP_PATH: &str = "maps/medium_plastic/scene.gltf";
pub const SPAWN_POINT_MEDIUM_PLASTIC_MAP: Vec3 = vec3(0.0, 10.0, 0.0);

/// Logic for both client and server binary
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);

        app.add_plugins(PhysicsPlugins::default().build())
            .insert_resource(Gravity(Vec3::NEG_Y * GRAVITY));
        app.add_systems(Update, handle_despawn_timer);
    }
}

// This is not a substate, as it needs to exist globally
#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
pub enum SelectedMapState {
    #[default]
    MediumPlastic,
    TinyTown,
}

pub fn handle_despawn_timer(
    despawn_timer_query: Query<(Entity, &mut DespawnTimer)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut timer) in despawn_timer_query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
