use bevy::prelude::*;

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[states(scoped_entities)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[states(scoped_entities)]
pub enum MainMenuState {
    #[default]
    Root,
    Settings,
    GameModeSelection,
}

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[states(scoped_entities)]
pub enum InGameState {
    #[default]
    Playing,
    Paused,
    PlayerDead,
    /// Everything is paused, no pause menu is shown, useful in combination with
    /// `PlayerCameraState::FreeCam`
    PausedDebug,
}

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[states(scoped_entities)]
pub enum AppDebugState {
    #[default]
    DebugHidden,
    DebugVisible,
}
