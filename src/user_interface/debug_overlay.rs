use bevy::prelude::*;
use bevy_inspector_egui::egui::TextStyle;

use crate::{
    game_flow::states::{AppDebugState, AppState, InGameState, MainMenuState},
    nav_mesh_pathfinding::NavMeshDisp,
    player::{Player, movement::PlayerMovementState},
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
                update_player_info_text,
                update_current_main_menu_state,
                update_player_movement_state_text,
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
struct PlayerInfoText;

#[derive(Component)]
struct PlayerMovementStateText;

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
            StateScoped(AppDebugState::DebugVisible),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                PlayerInfoText,
                TextFont {
                    font_size: DEBUG_OVERLAY_TEXT_SIZE,
                    ..default()
                },
            ));
            parent.spawn((
                Text::new("Player Movement State:"),
                TextFont {
                    font_size: DEBUG_OVERLAY_TEXT_SIZE,
                    ..default()
                },
            ));
            parent.spawn((
                Text::new(""),
                PlayerMovementStateText,
                TextFont {
                    font_size: DEBUG_OVERLAY_TEXT_SIZE,
                    ..default()
                },
            ));
            parent.spawn((
                Text::new("Current AppState:"),
                TextFont {
                    font_size: DEBUG_OVERLAY_TEXT_SIZE,
                    ..default()
                },
            ));
            parent.spawn((
                Text::new(""),
                CurrentAppStateText,
                TextFont {
                    font_size: DEBUG_OVERLAY_TEXT_SIZE,
                    ..default()
                },
            ));
            parent.spawn((
                Text::new("Current InGameState:"),
                TextFont {
                    font_size: DEBUG_OVERLAY_TEXT_SIZE,
                    ..default()
                },
            ));
            parent.spawn((
                (Text::new(""), CurrentInGameStateText),
                TextFont {
                    font_size: DEBUG_OVERLAY_TEXT_SIZE,
                    ..default()
                },
            ));
            parent.spawn((
                Text::new("Current MainMenuState:"),
                TextFont {
                    font_size: DEBUG_OVERLAY_TEXT_SIZE,
                    ..default()
                },
            ));
            parent.spawn((
                Text::new(""),
                CurrentMainMenuStateText,
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
        match *app_state.get() {
            AppState::MainMenu => {
                **current_app_state_text = Text::new("MainMenu");
            }
            AppState::InGame => {
                **current_app_state_text = Text::new("InGame");
            }
        }
    }
}

fn update_current_main_menu_state(
    mut current_main_menu_state_text: Single<
        &mut Text,
        With<CurrentMainMenuStateText>,
    >,
    main_menu_state: Res<State<MainMenuState>>,
) {
    if main_menu_state.is_changed() {
        match *main_menu_state.get() {
            MainMenuState::None => {
                **current_main_menu_state_text = Text::new("None");
            }
            MainMenuState::Root => {
                **current_main_menu_state_text = Text::new("Root");
            }
            MainMenuState::Settings => {
                **current_main_menu_state_text = Text::new("Settings");
            }
            MainMenuState::GameModeSelection => {
                **current_main_menu_state_text = Text::new("GameModeSelection");
            }
        }
    }
}

fn update_current_in_game_state_text(
    mut current_in_game_state_text: Single<
        &mut Text,
        With<CurrentInGameStateText>,
    >,
    in_game_state: Res<State<InGameState>>,
) {
    if in_game_state.is_changed() {
        match *in_game_state.get() {
            InGameState::Playing => {
                **current_in_game_state_text = Text::new("Playing");
            }
            InGameState::PlayerDead => {
                **current_in_game_state_text = Text::new("PlayerDead");
            }
            InGameState::Paused => {
                **current_in_game_state_text = Text::new("Paused");
            }
            InGameState::PausedDebug => {
                **current_in_game_state_text = Text::new("PausedDebug");
            }
            InGameState::None => {
                **current_in_game_state_text = Text::new("None");
            }
        }
    }
}

fn toggle_debug(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_app_debug_state: Res<State<AppDebugState>>,
    mut next_app_debug_state: ResMut<NextState<AppDebugState>>,
    nav_mesh_disp: Query<&mut Visibility, With<NavMeshDisp>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        match *current_app_debug_state.get() {
            AppDebugState::DebugHidden => {
                next_app_debug_state.set(AppDebugState::DebugVisible);
                for mut nav_mesh_disp in nav_mesh_disp {
                    *nav_mesh_disp = Visibility::Visible;
                }
            }
            AppDebugState::DebugVisible => {
                next_app_debug_state.set(AppDebugState::DebugHidden);
                for mut nav_mesh_disp in nav_mesh_disp {
                    *nav_mesh_disp = Visibility::Hidden;
                }
            }
        }
    }
}

fn update_player_info_text(
    changed_player: Single<&Player, Changed<Player>>,
    player_text: Query<&mut Text, With<PlayerInfoText>>,
) {
    for mut player_info_text in player_text {
        **player_info_text = format!("{:?}", *changed_player);
    }
}

fn update_player_movement_state_text(
    changed_player_movement_state: Single<
        &PlayerMovementState,
        Changed<PlayerMovementState>,
    >,
    player_movement_state_text: Query<&mut Text, With<PlayerMovementStateText>>,
) {
    for mut player_movement_state_text in player_movement_state_text {
        **player_movement_state_text =
            format!("{:?}", changed_player_movement_state.0);
    }
}
