use bevy::prelude::*;

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[states(scoped_entities)]
pub enum AppState {
    #[default]
    MainMenu,
    LoadingGame,
    InGame,
}

/// The current loading state of a new game.
/// Note that upon entering each of these states, the corresponding
/// systems will be run, e.g. SpawningMap state will spawn the map
#[derive(SubStates, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[source(AppState = AppState::LoadingGame)]
pub enum LoadingGameSubState {
    #[default]
    SpawningMap,
    MapLoadedWithDependencies,
    CollidersReady,
    NavMeshReady,
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

// This is not a substate, as it needs to exist globally
#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
pub enum SelectedMapState {
    #[default]
    MediumPlastic,
    TinyTown,
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

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[states(scoped_entities)]
pub enum AppDebugState {
    #[default]
    DebugHidden,
    DebugVisible,
}
