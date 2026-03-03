use avian3d::{PhysicsPlugins, prelude::*};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{components::DespawnTimer, protocol::ProtocolPlugin};

pub mod character_controller;
pub mod components;
pub mod enemy;
pub mod game_score;
pub mod player;
pub mod protocol;
pub mod shooting;
pub mod utils;

pub const DEFAULT_HEALTH: f32 = 100.0;

#[derive(Resource, PartialEq)]
pub enum ServerRunMode {
    Headless,
    Headful,
}

/// This state exists both in server binary and client binary. As game_core runs on both server
/// binary and client binary, it helps game_core to understand in which context it is currently
/// running. So for example if the game_core checks this state and sees ClientOnly, then it knows
/// not to run simulation logic, as the multiplayer server already runs that
#[derive(States, PartialEq, Debug, Hash, Clone, Eq)]
pub enum AppRole {
    /// This app is a client that is connecting to multiplayer server
    ClientOnly,
    /// This app is a client that is also hosting a local server, e.g. Singleplayer
    ClientAndServer,
    /// This app is the server binary
    DedicatedServer,
}

/// The game mode that is running on the server.
/// In case of AppRole::DedicatedServer, this gets replicated to all connected clients.
/// Must be a component as we can only replicate components with lightyear.
#[derive(Component, Serialize, Deserialize, PartialEq, Debug, Clone)]
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

#[derive(Component)]
pub struct Medkit {
    pub active: bool,
    pub health_to_give: f32,
    pub float_direction: MedkitFloatDirection,
    pub respawn_timer: Timer,
}

pub enum MedkitFloatDirection {
    Up,
    Down,
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

// A client can send this message to the server indicating that the player requested a respawn.
// The server will then update the players health and the players position.
#[derive(Serialize, Deserialize)]
pub struct ClientRespawnRequest;

/// This message is sent from server to client, so the client can spawn the damage indicator
#[derive(Serialize, Deserialize, Message, Copy, Clone)]
pub struct PlayerHitMessage {
    pub origin: Vec3,
}

// The server will send this message to the client that the respawn was made and the client can now
// update internal state, such as `InGameState`.
#[derive(Serialize, Deserialize)]
pub struct ConfirmRespawn;

/// 0: The enemy entity that was killed
#[derive(Message)]
pub struct EnemyKilledMessage(pub Entity);

pub const GRAVITY: f32 = 9.81;

pub const TINY_TOWN_MAP_PATH: &str = "maps/tiny_town/main.gltf";
pub const MEDIUM_PLASTIC_MAP_PATH: &str = "maps/medium_plastic/scene.gltf";
pub const SPAWN_POINT_MEDIUM_PLASTIC_MAP: Vec3 = vec3(0.0, 10.0, 0.0);

// FIXME: hmm thats weird. the below description is the same as GameCore? i think we should rather
// not have a SharedPlugin, and just do this stuff in game_core. hmm but adding
// ProtocolPlugin in game_core doesnt feel right.

/// Logic for both client and server binary
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);

        app.add_message::<StartGame>();
        app.add_message::<StopGame>();

        app.add_plugins(PhysicsPlugins::default().build())
            .insert_resource(Gravity(Vec3::NEG_Y * GRAVITY));
        app.add_systems(Update, handle_despawn_timer);
    }
}

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
pub enum CurrentMap {
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

/// game_core listens for this message. Upon receiving, game_core will spawn the map, spawn colliders,
/// spawn enemies, etc...
#[derive(Message)]
pub struct StartGame {
    pub map: CurrentMap,
    pub game_mode: GameModeServer,
}

/// A client can send this message to game_core, and game_core will despawn the map,
/// despawn enemies, etc
///  NOTE: This message gets ignored if game_core has AppRole::DedicatedServer (atm)
#[derive(Message)]
pub struct StopGame;
