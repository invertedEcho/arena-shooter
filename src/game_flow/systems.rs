use bevy::{prelude::*, window::CursorGrabMode};

use crate::{
    game_flow::{GameState, PlayerDeathEvent},
    player::{Player, PlayerState},
};

pub fn grab_mouse(mut window: Single<&mut Window>) {
    window.cursor_options.visible = false;
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
}

pub fn free_mouse(mut window: Single<&mut Window>) {
    window.cursor_options.visible = true;
    window.cursor_options.grab_mode = CursorGrabMode::None;
}

pub fn handle_player_death_event(
    mut event_reader: EventReader<PlayerDeathEvent>,
    mut player: Single<&mut Player>,
) {
    for _ in event_reader.read() {
        player.state = PlayerState::Idle;
    }
}

pub fn handle_escape(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match *current_game_state.get() {
            GameState::InGame => {
                next_game_state.set(GameState::Paused);
            }
            GameState::Paused => {
                next_game_state.set(GameState::InGame);
            }
            GameState::Settings => {
                next_game_state.set(GameState::Paused);
            }
            GameState::Death => {}
            GameState::MainMenu => {}
        }
    }
}
