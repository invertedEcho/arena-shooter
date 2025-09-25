use bevy::prelude::*;

use crate::game_flow::states::{AppState, InGameState};

pub struct DebugOverlayPlugin;

impl Plugin for DebugOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_debug_overlay).add_systems(
            Update,
            (
                update_current_app_state_text,
                update_current_in_game_state_text,
            ),
        );
    }
}

#[derive(Component)]
struct CurrentAppStateText;

#[derive(Component)]
struct CurrentInGameStateText;

fn spawn_debug_overlay(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Start,
            align_items: AlignItems::End,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(Text::new("Current AppState"));
            parent.spawn((Text::new(""), CurrentAppStateText));
            parent.spawn(Text::new("Current InGameState"));
            parent.spawn((Text::new(""), CurrentInGameStateText));
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
        }
    }
}
