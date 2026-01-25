use bevy::prelude::*;
use shared::GameModeServer;

use crate::game_flow::states::{
    AppDebugState, AppState, InGameState, LoadingGameState, MainMenuState,
};

const DEBUG_OVERLAY_TEXT_SIZE: f32 = 15.0;

pub struct DebugOverlayPlugin;

impl Plugin for DebugOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppDebugState::DebugVisible),
            spawn_debug_overlay,
        )
        .add_systems(
            Update,
            (
                update_current_app_state_text,
                update_current_in_game_state_text,
                toggle_debug,
                update_current_main_menu_state,
                update_loading_game_state_text,
                update_current_server_game_mode_text,
            ),
        );
    }
}

#[derive(Component)]
struct CurrentAppStateText;

#[derive(Component)]
struct CurrentInGameStateText;

#[derive(Component)]
struct CurrentMainMenuStateText;

#[derive(Component)]
struct CurrentServerGameModeText;

#[derive(Component)]
struct CurrentLoadingGameStateText;

fn spawn_debug_overlay(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::End,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            DespawnOnExit(AppDebugState::DebugVisible),
            Name::new("Debug Overlay UI Root"),
        ))
        .with_children(|parent| {
            // App State Text
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    ..default()
                })
                .with_child((
                    Text::new("Current AppState: "),
                    TextFont {
                        font_size: DEBUG_OVERLAY_TEXT_SIZE,
                        ..default()
                    },
                ))
                .with_child((
                    Text::new("None"),
                    CurrentAppStateText,
                    TextFont {
                        font_size: DEBUG_OVERLAY_TEXT_SIZE,
                        ..default()
                    },
                ));

            // InGame State Text
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    ..default()
                })
                .with_child((
                    Text::new("Current InGameState: "),
                    TextFont {
                        font_size: DEBUG_OVERLAY_TEXT_SIZE,
                        ..default()
                    },
                ))
                .with_child((
                    Text::new("None"),
                    CurrentInGameStateText,
                    TextFont {
                        font_size: DEBUG_OVERLAY_TEXT_SIZE,
                        ..default()
                    },
                ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    ..default()
                })
                .with_child((
                    Text::new("Current MainMenuState: "),
                    TextFont {
                        font_size: DEBUG_OVERLAY_TEXT_SIZE,
                        ..default()
                    },
                ))
                .with_child((
                    Text::new("None"),
                    CurrentMainMenuStateText,
                    TextFont {
                        font_size: DEBUG_OVERLAY_TEXT_SIZE,
                        ..default()
                    },
                ));
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    ..default()
                })
                .with_child((
                    Text::new("Current ServerGameMode: "),
                    TextFont {
                        font_size: DEBUG_OVERLAY_TEXT_SIZE,
                        ..default()
                    },
                ))
                .with_child((
                    Text::new("None"),
                    CurrentServerGameModeText,
                    TextFont {
                        font_size: DEBUG_OVERLAY_TEXT_SIZE,
                        ..default()
                    },
                ));
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    ..default()
                })
                .with_child((
                    Text::new("Current LoadingGameState: "),
                    TextFont {
                        font_size: DEBUG_OVERLAY_TEXT_SIZE,
                        ..default()
                    },
                ))
                .with_child((
                    Text::new("None"),
                    CurrentLoadingGameStateText,
                    TextFont {
                        font_size: DEBUG_OVERLAY_TEXT_SIZE,
                        ..default()
                    },
                ));
        });
}

fn update_current_app_state_text(
    mut current_app_state_text: Single<&mut Text, With<CurrentAppStateText>>,
    app_state: Res<State<AppState>>,
) {
    if app_state.is_changed() {
        **current_app_state_text = Text::new(format!("{:?}", *app_state.get()));
    }
}

fn update_current_main_menu_state(
    mut current_main_menu_state_text: Single<
        &mut Text,
        With<CurrentMainMenuStateText>,
    >,
    main_menu_state: Option<Res<State<MainMenuState>>>,
) {
    if let Some(main_menu_state) = main_menu_state {
        if main_menu_state.is_changed() {
            **current_main_menu_state_text =
                Text::new(format!("{:?}", *main_menu_state.get()));
        }
    } else {
        **current_main_menu_state_text =
            Text::new("MainMenuState doesn't exist");
    }
}

fn update_current_in_game_state_text(
    mut current_in_game_state_text: Single<
        &mut Text,
        With<CurrentInGameStateText>,
    >,
    in_game_state: Option<Res<State<InGameState>>>,
) {
    if let Some(in_game_state) = in_game_state {
        if in_game_state.is_changed() {
            **current_in_game_state_text =
                Text::new(format!("{:?}", *in_game_state.get()));
        }
    } else {
        **current_in_game_state_text = Text::new("InGameState doesn't exist");
    }
}

fn update_current_server_game_mode_text(
    mut current_server_game_mode_text: Single<
        &mut Text,
        With<CurrentServerGameModeText>,
    >,
    server_game_mode: Single<&GameModeServer, Changed<GameModeServer>>,
) {
    **current_server_game_mode_text =
        Text::new(format!("{:?}", *server_game_mode));
}

fn update_loading_game_state_text(
    mut current_loading_game_state_text: Single<
        &mut Text,
        With<CurrentLoadingGameStateText>,
    >,
    loading_game_state: Option<Res<State<LoadingGameState>>>,
) {
    let Some(loading_game_state) = loading_game_state else {
        **current_loading_game_state_text =
            Text::new("LoadingGameState doesn't exist");
        return;
    };
    if loading_game_state.is_changed() {
        **current_loading_game_state_text =
            Text::new(format!("{:?}", *loading_game_state));
    }
}

fn toggle_debug(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_app_debug_state: Res<State<AppDebugState>>,
    mut next_app_debug_state: ResMut<NextState<AppDebugState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        match *current_app_debug_state.get() {
            AppDebugState::DebugHidden => {
                next_app_debug_state.set(AppDebugState::DebugVisible);
            }
            AppDebugState::DebugVisible => {
                next_app_debug_state.set(AppDebugState::DebugHidden);
            }
        }
    }
}
