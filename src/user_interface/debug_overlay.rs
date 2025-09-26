use bevy::prelude::*;

use crate::{
    game_flow::states::{AppDebugState, AppState, InGameState},
    nav_mesh_pathfinding::NavMeshDisp,
};

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
