use bevy::prelude::*;

use crate::game_flow::{
    game_mode::GameModePlugin,
    score::GameScorePlugin,
    systems::{
        free_mouse, grab_mouse, handle_enter_in_game_state, handle_escape,
        handle_exit_in_game_state, handle_player_death_event,
    },
};

pub mod game_mode;
pub mod score;
pub mod systems;

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameScorePlugin)
            .add_plugins(GameModePlugin)
            .add_event::<PlayerDeathEvent>()
            .add_systems(OnEnter(AppState::InGame), grab_mouse)
            .add_systems(OnEnter(AppState::PauseMenu), free_mouse)
            .add_systems(Update, (handle_escape, handle_player_death_event))
            .add_systems(OnEnter(AppState::InGame), handle_enter_in_game_state)
            .add_systems(OnExit(AppState::InGame), handle_exit_in_game_state)
            .init_state::<AppState>();
    }
}

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[states(scoped_entities)]
pub enum AppState {
    #[default]
    MainMenu,
    GameModeSelection,
    InGame,
    PauseMenu,
    // TODO: i dont think this should be an AppState
    PlayerDead,
    SettingsMainMenu,
    SettingsPauseMenu,
}

#[derive(Event)]
pub struct PlayerDeathEvent;
