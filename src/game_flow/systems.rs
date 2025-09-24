use bevy::{prelude::*, window::CursorGrabMode};

use crate::{
    game_flow::{AppState, PlayerDeathEvent},
    player::{Player, PlayerState, shooting::components::PlayerWeapon},
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
    current_app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match *current_app_state.get() {
            AppState::InGame => {
                next_app_state.set(AppState::PauseMenu);
            }
            AppState::PauseMenu => {
                next_app_state.set(AppState::InGame);
            }
            AppState::SettingsPauseMenu => {
                next_app_state.set(AppState::PauseMenu);
            }
            AppState::PlayerDead => {}
            AppState::MainMenu => {}
            AppState::GameModeSelection => {
                next_app_state.set(AppState::MainMenu);
            }
            AppState::SettingsMainMenu => {}
        }
    }
}

pub fn handle_enter_in_game_state(
    mut player_weapon: Single<&mut Visibility, With<PlayerWeapon>>,
) {
    **player_weapon = Visibility::Visible;
}

pub fn handle_exit_in_game_state(
    mut player_weapon: Single<&mut Visibility, With<PlayerWeapon>>,
) {
    **player_weapon = Visibility::Hidden;
}
