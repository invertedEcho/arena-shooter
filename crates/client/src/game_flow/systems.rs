use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use game_core::RequestNewWave;
use lightyear::prelude::MessageSender;
use shared::{
    GameStateServer, multiplayer_messages::ChangeGameServerStateRequest,
    protocol::OrderedReliableChannel,
};

use crate::{
    game_flow::states::{AppState, InGameState},
    player::{
        PlayerDeathMessage,
        camera::{components::MainMenuCamera, get_main_menu_camera_transform},
    },
};

pub fn grab_mouse(
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    primary_cursor_options.visible = false;
    primary_cursor_options.grab_mode = CursorGrabMode::Locked;
}

pub fn free_mouse(
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    primary_cursor_options.visible = true;
    primary_cursor_options.grab_mode = CursorGrabMode::None;
}

pub fn manual_mouse_grab_toggle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyU) {
        if primary_cursor_options.grab_mode == CursorGrabMode::None {
            primary_cursor_options.grab_mode = CursorGrabMode::Locked;
        } else {
            primary_cursor_options.grab_mode = CursorGrabMode::None;
        }
    }
}

pub fn handle_escape_in_game(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_in_game_state: If<Res<State<InGameState>>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    let escape_just_pressed = keyboard_input.just_pressed(KeyCode::Escape);
    let current_in_game_state = current_in_game_state.get();

    if escape_just_pressed {
        match current_in_game_state {
            InGameState::Playing => {
                next_in_game_state.set(InGameState::Paused);
            }
            InGameState::Paused => next_in_game_state.set(InGameState::Playing),
            InGameState::PlayerDead => {}
        }
    }
}

pub fn spawn_main_menu_camera(
    mut commands: Commands,
    existing_main_menu_camera: Query<&MainMenuCamera>,
) {
    // TODO: optimally this couldnt happen in first place
    if existing_main_menu_camera.count() != 0 {
        debug!("Not spawning Main Menu Camera, already exists");
        return;
    }

    debug!("Spawning Main Menu Camera");
    let main_menu_camera = commands
        .spawn((
            Name::new("Main Menu Camera"),
            Camera::default(),
            Camera3d::default(),
            get_main_menu_camera_transform(),
            MainMenuCamera,
            // we still need main menu camera during loading screen
            DespawnOnEnter(AppState::InGame),
        ))
        .id();

    if cfg!(debug_assertions) {
        commands
            .entity(main_menu_camera)
            .insert(bevy_inspector_egui::bevy_egui::PrimaryEguiContext);
    }
}

pub fn pause_all_animations(animation_players: Query<&mut AnimationPlayer>) {
    for mut animation_player in animation_players {
        animation_player.pause_all();
    }
}

pub fn resume_all_animations(animation_players: Query<&mut AnimationPlayer>) {
    for mut animation_player in animation_players {
        animation_player.resume_all();
    }
}

pub fn handle_player_death_event(
    mut message_reader: MessageReader<PlayerDeathMessage>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    for _ in message_reader.read() {
        next_in_game_state.set(InGameState::PlayerDead);
    }
}

pub fn send_update_game_server_state_request_on_in_game_state_change(
    current_in_game_state: If<Res<State<InGameState>>>,
    mut message_sender: Single<
        &mut MessageSender<ChangeGameServerStateRequest>,
    >,
) {
    match *current_in_game_state.get() {
        InGameState::Playing => {
            message_sender.send::<OrderedReliableChannel>(
                ChangeGameServerStateRequest(GameStateServer::Running),
            );
        }
        InGameState::Paused | InGameState::PlayerDead => {
            message_sender.send::<OrderedReliableChannel>(
                ChangeGameServerStateRequest(GameStateServer::Paused),
            );
        }
    }
}

pub fn handle_request_next_wave(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut message_writer: MessageWriter<RequestNewWave>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        message_writer.write(RequestNewWave);
    }
}
