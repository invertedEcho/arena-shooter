use bevy::prelude::*;

use crate::{
    game_flow::{
        game_mode::GameModePlugin,
        score::GameScorePlugin,
        states::{AppDebugState, AppState, InGameState, MainMenuState},
        systems::{
            free_mouse, grab_mouse, handle_escape, handle_exit_in_game,
            restart_game, spawn_main_menu_camera,
        },
    },
    player::PlayerDeathEvent,
};

pub mod game_mode;
pub mod score;
pub mod states;
pub mod systems;

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_state::<InGameState>()
            .init_state::<MainMenuState>()
            .init_state::<AppDebugState>()
            .add_event::<PlayerDeathEvent>()
            .add_plugins(GameScorePlugin)
            .add_plugins(GameModePlugin)
            .add_systems(OnEnter(InGameState::Playing), grab_mouse)
            .add_systems(OnEnter(InGameState::Paused), free_mouse)
            .add_systems(OnExit(AppState::InGame), handle_exit_in_game)
            .add_systems(Update, handle_escape)
            .add_systems(Startup, spawn_main_menu_camera)
            .add_systems(Update, restart_game);
    }
}
