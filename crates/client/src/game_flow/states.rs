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
#[derive(SubStates, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[source(AppState = AppState::LoadingGame)]
pub enum ClientLoadingState {
    /// This state is set when the client is starting local server to play Singleplayer on
    /// It is skipped in case of connecting to the official dedicated server, e.g. multiplayer
    #[default]
    StartingServer,
    /// This state is set when the client connects to the game server
    ConnectingToServer,
}

impl Display for ClientLoadingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StartingServer => f.write_str("Starting local server"),
            Self::ConnectingToServer => f.write_str("Connecting to the server"),
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
    Credits,
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

// The current game mode on the client
#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default, Copy)]
pub enum GameModeClient {
    #[default]
    FreeRoam,
    Waves,
    Multiplayer,
}
