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
    // TODO: can we then just get rid of this state alltogether?
    // i mean StartingServer, thats just done via game_core, not client
    // and then we are only left with ConnectingToServer, and one state means we can just have
    // AppState::Loading / ConnectingToGameServer
    /// Note that this state will be skipped if AppRole::ClientOnly. This is only relevant for
    /// AppRole::ClientAndServer
    #[default]
    StartingServer,
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
