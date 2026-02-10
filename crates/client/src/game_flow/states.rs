use bevy::prelude::*;
use std::fmt::Display;

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[states(scoped_entities)]
pub enum AppState {
    #[default]
    MainMenu,
    LoadingGame,
    InGame,
    Disconnected,
}

/// The current loading state of the client.
/// Note that upon entering each of these states, the corresponding
/// systems will be run, e.g. SpawningMap state will spawn the map
#[derive(SubStates, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[source(AppState = AppState::LoadingGame)]
pub enum ClientLoadingState {
    #[default]
    ConnectingToServer,
    SpawningMap,
    SpawningColliders,
}

impl Display for ClientLoadingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SpawningMap => f.write_str("Spawning the map"),
            Self::SpawningColliders => f.write_str("Setting up collisions"),
            Self::ConnectingToServer => {
                f.write_str("Connecting to the game server")
            }
        }
    }
}

#[derive(SubStates, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[states(scoped_entities)]
#[source(AppState = AppState::MainMenu)]
pub enum MainMenuState {
    #[default]
    Root,
    Settings,
    MapSelection,
    GameModeSelection,
}

#[derive(SubStates, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[states(scoped_entities)]
#[source(AppState = AppState::InGame)]
pub enum InGameState {
    #[default]
    Playing,
    Paused,
    PlayerDead,
}

/// The current connection state to the game server.
#[derive(States, Debug, Hash, Eq, PartialEq, Clone, Default)]
pub enum ConnectionState {
    #[default]
    Connecting,
    Connected,
    Disconnected,
}

// The current game mode on the client
#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default, Copy)]
pub enum GameModeClient {
    #[default]
    FreeRoam,
    Waves,
    Multiplayer,
}
