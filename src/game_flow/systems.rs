use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    enemy::Enemy,
    game_flow::{
        AppState,
        game_mode::{GameMode, StartGameModeMessage},
        states::InGameState,
    },
    player::{Player, camera::components::FreeCam},
    user_interface::main_menu::MainMenuCamera,
};

pub fn grab_mouse(
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    info!("Grabbing mouse");
    primary_cursor_options.visible = false;
    primary_cursor_options.grab_mode = CursorGrabMode::Locked;
}

pub fn free_mouse(
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    primary_cursor_options.visible = true;
    primary_cursor_options.grab_mode = CursorGrabMode::None;
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
    mut start_game_mode_message_writer: MessageWriter<StartGameModeMessage>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        info!("Restarting game");
        start_game_mode_message_writer
            .write(StartGameModeMessage(GameMode::Waves));
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
    free_cam_cameras: Query<Entity, With<FreeCam>>,
) {
    info!("Despawning all enemies");
    for enemy in enemies {
        commands.entity(enemy).despawn();
    }
    info!("Despawning all players");
    for player in players {
        commands.entity(player).despawn();
    }
    for free_cam_camera in free_cam_cameras {
        info!("Despawning free cam");
        commands.entity(free_cam_camera).despawn();
    }

    info!("Spawning Main Menu Camera");
    commands.spawn((
        Camera::default(),
        Camera3d::default(),
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainMenuCamera,
    ));
}
