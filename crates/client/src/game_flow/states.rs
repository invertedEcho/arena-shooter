use bevy::prelude::*;
use shared::GameMode;
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

// TODO
/// The current loading state of the client.
/// Note that it only really has connection states, because the client itself only connects to the
/// server. Map spawning etc is done by game_core. (which basically also runs in client... hmm)
#[derive(SubStates, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[source(AppState = AppState::LoadingGame)]
pub enum ClientLoadingState {
    /// This state is set when the client is starting local server to play Singleplayer on
    /// It is skipped in case of connecting to the official dedicated server, e.g. multiplayer
    StartingServer,
    /// This state is set when the client connects to the game server
    #[default]
    ConnectingToServer,
    ConnectedToServer,
}

impl Display for ClientLoadingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StartingServer => f.write_str("Starting local server"),
            Self::ConnectingToServer => f.write_str("Connecting to the server"),
            Self::ConnectedToServer => f.write_str("Connected to server!"),
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

/// The current "configuration" of the game, on the client. It stores intermediate values such as
/// game mode selected, map selected, until this gets sent to the server/game_core.
#[derive(Resource, Debug, Default)]
pub struct GameConfigClient {
    pub game_mode: GameMode,
}
