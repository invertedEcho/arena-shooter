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

/// A struct describing the current configuratioan of the running game.
/// On the client,
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct GameConfig {
    pub game_mode: GameMode,
    pub game_map: GameMap,
}

#[derive(Resource, PartialEq, Debug)]
pub enum ServerRunMode {
    Headless,
    Headful,
}

/// This state exists to help game_core to understand in which context it is currently
/// running. For example, if the game_core checks this state and sees ClientOnly, then it knows
/// not to run simulation logic, as the dedicated server already runs that
#[derive(States, PartialEq, Debug, Hash, Clone, Eq)]
pub enum AppRole {
    /// This app is a client that is connecting to multiplayer server
    ClientOnly,
    /// This app is a client that is also hosting a local server, e.g. Singleplayer
    HostClient,
    /// This app is the server binary
    DedicatedServer,
}

/// The currently active game configuration on the server
#[derive(Resource, Serialize, Deserialize)]
pub struct GameConfigServer(pub GameConfig);

#[derive(
    States, Serialize, Deserialize, PartialEq, Clone, Eq, Hash, Debug, Default,
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

pub const TINY_TOWN_MAP_PATH: &str = "maps/tiny_town/scene.gltf";
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub enum GameMap {
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

#[derive(Message, Copy, Clone)]
pub struct StartGame(pub GameConfig);

/// A client can send this message to game_core, and game_core will despawn the map,
/// despawn enemies, etc
///  NOTE: This message gets ignored if game_core has AppRole::DedicatedServer (atm)
#[derive(Message)]
pub struct StopGame;

/// When this timer finishes, the next wave gets spawned.
/// It gets inserted when finishing a wave, and removed when the timer has finished / when the next
/// wave has started.
#[derive(Resource)]
pub struct NextWaveTimer(pub Timer);

/// This message gets written whenever the current wave is finished, e.g. all enemies of the
/// corresponding wave are killed
#[derive(Message)]
pub struct WaveFinishedMessage;

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone, Default)]
pub enum GameMode {
    /// Every wave more enemies get spawned
    Waves,
    /// Player can move around the map freely
    #[default]
    FreeRoam,
    /// Every player against every other player, no teams
    FreeForAll,
}
