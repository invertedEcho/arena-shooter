use bevy::prelude::*;

use crate::{
    game_flow::{
        game_mode::GameModePlugin,
        score::GameScorePlugin,
        states::{
            AppDebugState, AppState, InGameState, LoadingGameSubState,
            MainMenuState, SelectedMapState,
        },
        systems::{
            check_collider_constructor_hierarchy_ready,
            check_world_scene_loaded, free_mouse, grab_mouse,
            handle_escape_in_game, manual_free_mouse,
            on_game_loading_state_nav_mesh_ready, pause_all_animations,
            resume_all_animations, spawn_main_menu_camera,
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
            .init_state::<AppDebugState>()
            .init_state::<SelectedMapState>()
            .add_sub_state::<InGameState>()
            .add_sub_state::<MainMenuState>()
            .add_sub_state::<LoadingGameSubState>()
            .add_message::<PlayerDeathMessage>()
            .add_plugins(GameScorePlugin)
            .add_plugins(GameModePlugin)
            .add_observer(check_collider_constructor_hierarchy_ready)
            // .add_observer(check_navmesh_ready)
            .add_systems(
                OnEnter(LoadingGameSubState::NavMeshReady),
                on_game_loading_state_nav_mesh_ready,
            )
            .add_systems(
                OnEnter(InGameState::Playing),
                (grab_mouse, resume_all_animations),
            )
            .add_systems(
                OnEnter(InGameState::Paused),
                (free_mouse, pause_all_animations),
            )
            .add_systems(OnEnter(AppState::MainMenu), spawn_main_menu_camera)
            .add_systems(
                Update,
                (
                    check_world_scene_loaded,
                    handle_escape_in_game,
                    manual_free_mouse,
                ),
            )
            .add_systems(
                Update,
                check_world_scene_loaded
                    .run_if(resource_added::<WorldSceneHandle>),
            );
    }
}
