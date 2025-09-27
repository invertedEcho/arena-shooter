use bevy::prelude::*;

use crate::{
    game_flow::{
        game_mode::GameModePlugin,
        score::GameScorePlugin,
        states::{AppDebugState, AppState, InGameState, MainMenuState},
        systems::{
            enable_debug_paused, free_mouse, grab_mouse, handle_escape,
            handle_player_death_event, make_player_weapon_hidden,
            make_player_weapon_visible,
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
            .add_systems(OnEnter(AppState::InGame), grab_mouse)
            .add_systems(
                OnEnter(InGameState::Playing),
                (grab_mouse).run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnEnter(InGameState::Paused), free_mouse)
            .add_systems(Update, (handle_escape, handle_player_death_event))
            .add_systems(
                OnEnter(InGameState::Playing),
                make_player_weapon_visible,
            )
            .add_systems(
                OnExit(InGameState::Playing),
                make_player_weapon_hidden,
            )
            .add_systems(Update, (enable_debug_paused));
    }
}
