use bevy::prelude::*;

// TODO: do we even need this if we have the two states below?
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
    None,
    #[default]
    Root,
    Settings,
    GameModeSelection,
}

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[states(scoped_entities)]
pub enum InGameState {
    #[default]
    None,
    Playing,
    Paused,
    PlayerDead,
    /// Everything is paused, no pause menu is shown, useful in combination with
    PausedDebug,
}

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[states(scoped_entities)]
pub enum AppDebugState {
    DebugHidden,
    #[default]
    DebugVisible,
}
