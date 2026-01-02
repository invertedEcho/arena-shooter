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
    /// The map is being spawn
    #[default]
    SpawningMap,
    /// The map is loaded with all its dependencies
    MapLoadedWithDependencies,
    /// Avian generated all colliders for the map
    CollidersReady,
    /// Navmesh generation succeeded and the navmesh is ready
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
