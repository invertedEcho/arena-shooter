use bevy::{prelude::*, window::CursorGrabMode};

use crate::{
    game_flow::{AppState, states::InGameState},
    player::Player,
    user_interface::main_menu::MainMenuCamera,
};

pub fn grab_mouse(mut window: Single<&mut Window>) {
    window.cursor_options.visible = false;
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
}

pub fn free_mouse(mut window: Single<&mut Window>) {
    window.cursor_options.visible = true;
    window.cursor_options.grab_mode = CursorGrabMode::None;
}

pub fn handle_escape(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_app_state: Res<State<AppState>>,
    current_in_game_state: Res<State<InGameState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match *current_app_state.get() {
            AppState::InGame => match *current_in_game_state.get() {
                InGameState::None => {}
                InGameState::Playing => {
                    next_in_game_state.set(InGameState::Paused);
                }
                InGameState::PlayerDead => {}
                InGameState::Paused => {
                    next_in_game_state.set(InGameState::Playing)
                }
                InGameState::PausedDebug => {
                    next_in_game_state.set(InGameState::Playing);
                }
            },
            AppState::MainMenu => {}
        }
    }
}

// pub fn make_player_weapon_visible(
//     mut player_weapon: Single<&mut Visibility, With<PlayerWeapon>>,
// ) {
//     **player_weapon = Visibility::Visible;
// }
//
// pub fn make_player_weapon_hidden(
//     mut player_weapon: Single<&mut Visibility, With<PlayerWeapon>>,
// ) {
//     **player_weapon = Visibility::Hidden;
// }

pub fn enable_debug_paused(
    mut next_in_game_state: ResMut<NextState<InGameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        next_in_game_state.set(InGameState::PausedDebug);
    }
}

pub fn reset_player_position(
    mut player_transform: Single<&mut Transform, With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        player_transform.translation = Vec3::ZERO;
    }
}

pub fn spawn_main_menu_camera(mut commands: Commands) {
    commands.spawn((
        Camera::default(),
        Camera3d::default(),
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainMenuCamera,
    ));
}

pub fn handle_enter_main_menu(
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
) {
    info!("handling enter main menu");
    commands.entity(*player).despawn();

    commands.spawn((
        Camera::default(),
        Camera3d::default(),
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainMenuCamera,
    ));
}
