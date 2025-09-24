use bevy::prelude::*;

use crate::game_flow::AppState;

pub struct DebugOverlayPlugin;

impl Plugin for DebugOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_debug_overlay)
            .add_systems(Update, update_current_app_state_text);
    }
}

#[derive(Component)]
struct CurrentAppStateText;

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
            AppState::SettingsPauseMenu => {
                **current_app_state_text = Text::new("SettingsPauseMenu");
            }
            AppState::InGame => {
                **current_app_state_text = Text::new("InGame");
            }
            AppState::GameModeSelection => {
                **current_app_state_text = Text::new("GameModeSelection");
            }
            AppState::PlayerDead => {
                **current_app_state_text = Text::new("Death");
            }
            AppState::SettingsMainMenu => {
                **current_app_state_text = Text::new("SettingsMainMenu");
            }
            AppState::PauseMenu => {
                **current_app_state_text = Text::new("PauseMenu");
            }
        }
    }
}
