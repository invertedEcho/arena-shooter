use bevy::prelude::*;

use crate::{
    character_controller::components::Grounded,
    game_flow::states::{AppDebugState, AppState, InGameState, MainMenuState},
    player::Player,
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
                update_grounded_player,
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
struct PlayerGroundedText;

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
            parent.spawn(Text::new("Player Grounded"));
            parent.spawn((Text::new(""), PlayerGroundedText));
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

fn update_player_info_text(
    changed_player: Single<&Player, Changed<Player>>,
    player_text: Query<&mut Text, With<PlayerInfoText>>,
) {
    for mut player_info_text in player_text {
        **player_info_text = format!("{:?}", *changed_player);
    }
}

fn update_grounded_player(
    query: Single<&Grounded, (Changed<Grounded>, With<Player>)>,
    mut text_query: Single<&mut Text, With<PlayerGroundedText>>,
) {
    text_query.0 = query.0.to_string();
}
