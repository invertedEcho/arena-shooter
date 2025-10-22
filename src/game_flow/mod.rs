use bevy::prelude::*;

use crate::{
    game_flow::{
        game_mode::GameModePlugin,
        score::GameScorePlugin,
        states::{
            AppDebugState, AppState, GameLoadingState, InGameState,
            MainMenuState,
        },
        systems::{
            check_world_scene_loaded, free_mouse, grab_mouse, handle_escape,
            handle_exit_in_game, restart_game, spawn_main_menu_camera,
        },
    },
    player::PlayerDeathMessage,
    world::resources::WorldSceneHandle,
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
            .init_state::<GameLoadingState>()
            .add_message::<PlayerDeathMessage>()
            .add_plugins(GameScorePlugin)
            .add_plugins(GameModePlugin)
            .add_systems(OnEnter(InGameState::Playing), grab_mouse)
            .add_systems(OnEnter(InGameState::Paused), free_mouse)
            .add_systems(OnExit(AppState::InGame), handle_exit_in_game)
            .add_systems(Startup, spawn_main_menu_camera)
            .add_systems(
                Update,
                (restart_game, check_world_scene_loaded, handle_escape),
            )
            .add_systems(
                Update,
                check_world_scene_loaded
                    .run_if(resource_added::<WorldSceneHandle>),
            );
    }
}
