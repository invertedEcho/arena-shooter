use bevy::prelude::*;
use game_core::RequestNewWave;

use crate::{
    game_flow::{
        states::{
            AppState, ClientLoadingState, GameModeClient, InGameState,
            MainMenuState,
        },
        systems::{
            free_mouse, grab_mouse, handle_escape_in_game,
            handle_player_death_event, handle_request_next_wave,
            manual_mouse_grab_toggle, pause_all_animations,
            resume_all_animations,
            send_update_game_server_state_request_on_in_game_state_change,
            spawn_main_menu_camera,
        },
    },
    player::PlayerDeathMessage,
};

pub mod states;
pub mod systems;

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_state::<GameModeClient>()
            .add_sub_state::<InGameState>()
            .add_sub_state::<MainMenuState>()
            .add_sub_state::<ClientLoadingState>()
            .add_message::<PlayerDeathMessage>()
            .add_message::<RequestNewWave>()
            .add_systems(
                OnEnter(InGameState::Playing),
                (grab_mouse, resume_all_animations),
            )
            .add_systems(
                OnEnter(InGameState::Paused),
                (free_mouse, pause_all_animations),
            )
            .add_systems(OnEnter(AppState::MainMenu), free_mouse)
            .add_systems(OnEnter(AppState::MainMenu), spawn_main_menu_camera)
            .add_systems(
                Update,
                (
                    handle_escape_in_game,
                    manual_mouse_grab_toggle,
                    handle_player_death_event,
                ),
            )
            .add_systems(
                Update,
                send_update_game_server_state_request_on_in_game_state_change
                    .run_if(state_changed::<InGameState>),
            )
            .add_systems(
                Update,
                handle_request_next_wave
                    .run_if(in_state(GameModeClient::Waves)),
            );
    }
}
