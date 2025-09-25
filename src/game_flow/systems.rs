use bevy::{prelude::*, window::CursorGrabMode};

use crate::{
    game_flow::{AppState, PlayerDeathEvent, states::InGameState},
    player::{Player, PlayerMovementState, shooting::components::PlayerWeapon},
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
        player.state = PlayerMovementState::Idle;
    }
}

pub fn handle_escape(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    current_in_game_state: Res<State<InGameState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match *current_app_state.get() {
            AppState::InGame => match *current_in_game_state.get() {
                InGameState::Playing => {
                    next_in_game_state.set(InGameState::Paused);
                }
                InGameState::PlayerDead => {}
                InGameState::Paused => {
                    next_in_game_state.set(InGameState::Playing)
                }
            },
            AppState::MainMenu => {}
        }
    }
}

pub fn make_player_weapon_visible(
    mut player_weapon: Single<&mut Visibility, With<PlayerWeapon>>,
) {
    **player_weapon = Visibility::Visible;
}

pub fn make_player_weapon_hidden(
    mut player_weapon: Single<&mut Visibility, With<PlayerWeapon>>,
) {
    **player_weapon = Visibility::Hidden;
}
