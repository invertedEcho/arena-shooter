use bevy::{prelude::*, window::CursorGrabMode};

use crate::{
    enemy::Enemy,
    game_flow::{
        AppState,
        game_mode::{GameMode, StartGameModeEvent},
        states::InGameState,
    },
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
            },
            AppState::MainMenu => {}
        }
    }
}

// pub fn enable_debug_paused(
//     mut next_in_game_state: ResMut<NextState<InGameState>>,
//     keyboard_input: Res<ButtonInput<KeyCode>>,
// ) {
//     if keyboard_input.just_pressed(KeyCode::KeyP) {
//         next_in_game_state.set(InGameState::PausedDebug);
//     }
// }
//
// pub fn reset_player_position(
//     mut player_transform: Single<&mut Transform, With<Player>>,
//     keyboard_input: Res<ButtonInput<KeyCode>>,
// ) {
//     if keyboard_input.just_pressed(KeyCode::KeyP) {
//         player_transform.translation = Vec3::ZERO;
//     }
// }

pub fn restart_game(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut start_game_mode_writer: EventWriter<StartGameModeEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        info!("restarting game mode");
        start_game_mode_writer.write(StartGameModeEvent(GameMode::Waves));
    }
}

pub fn spawn_main_menu_camera(mut commands: Commands) {
    info!("Spawning Main Menu Camera");
    commands.spawn((
        Camera::default(),
        Camera3d::default(),
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainMenuCamera,
    ));
}

pub fn handle_exit_in_game(
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
    players: Query<Entity, (With<Player>, Without<Enemy>)>,
) {
    info!("Despawning all enemies");
    for enemy in enemies {
        commands.entity(enemy).despawn();
    }
    info!("Despawning all players");
    for player in players {
        commands.entity(player).despawn();
    }

    info!("Spawning Main Menu Camera");
    commands.spawn((
        Camera::default(),
        Camera3d::default(),
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainMenuCamera,
    ));
}
