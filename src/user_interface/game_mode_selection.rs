use bevy::prelude::*;

use crate::game_flow::{
    game_mode::{GameMode, StartGameModeEvent},
    states::{AppState, MainMenuState},
};

pub struct GameModeSelectionUIPlugin;

impl Plugin for GameModeSelectionUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MainMenuState::GameModeSelection),
            spawn_game_mode_selection_screen,
        )
        .add_systems(
            OnExit(MainMenuState::GameModeSelection),
            despawn_game_mode_selection_screen,
        )
        .add_systems(
            OnExit(AppState::MainMenu),
            despawn_game_mode_selection_screen,
        )
        .add_systems(
            Update,
            (
                handle_game_mode_selection_button_press,
                handle_game_mode_selection_action_button_press,
            ),
        );
    }
}

#[derive(Component)]
struct GameModeSelectionScreen;

#[derive(Component)]
struct GameModeSelectionButton(GameMode);

#[derive(Component)]
struct GameModeSelectionActionButton(GameModeSelectionActionButtonType);

enum GameModeSelectionActionButtonType {
    GoBack,
}

fn spawn_game_mode_selection_screen(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            GameModeSelectionScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    padding: UiRect::new(
                        Val::ZERO,
                        Val::ZERO,
                        Val::ZERO,
                        Val::Px(16.0),
                    ),
                    ..default()
                })
                .with_child(Text::new("Select a game mode"));
            parent
                .spawn((
                    Node { ..default() },
                    Button,
                    GameModeSelectionButton(GameMode::Waves),
                    TextColor::WHITE,
                ))
                .with_child(Text::new("Waves"));
            parent
                .spawn((
                    Node { ..default() },
                    Button,
                    GameModeSelectionButton(GameMode::FreePlay),
                    TextColor::WHITE,
                ))
                .with_child(Text::new("Free Play"));
            parent
                .spawn((
                    Node {
                        padding: UiRect {
                            top: Val::Px(16.0),
                            ..default()
                        },
                        ..default()
                    },
                    Button,
                    GameModeSelectionActionButton(
                        GameModeSelectionActionButtonType::GoBack,
                    ),
                    TextColor::WHITE,
                ))
                .with_child(Text::new("Go back"));
        });
}

fn despawn_game_mode_selection_screen(
    mut commands: Commands,
    game_mode_selection_screen: Single<Entity, With<GameModeSelectionScreen>>,
) {
    commands.entity(*game_mode_selection_screen).despawn();
}

fn handle_game_mode_selection_button_press(
    query: Query<
        (&Interaction, &GameModeSelectionButton),
        Changed<Interaction>,
    >,
    mut next_game_mode_state: ResMut<NextState<GameMode>>,
    mut event_writer: EventWriter<StartGameModeEvent>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    // honestly i really have the feeling all this game mode logic can be simplified,
    // like we should just watch the current game mode state annd update stuff when its changed
    for (interaction, game_mode_selection_button) in query {
        if let Interaction::Pressed = interaction {
            match game_mode_selection_button.0 {
                GameMode::Waves => {
                    next_app_state.set(AppState::InGame);
                    next_game_mode_state.set(GameMode::Waves);
                    event_writer.write(StartGameModeEvent(GameMode::Waves));
                }
                GameMode::FreePlay => {
                    info!("ineraction pressed on free play mode");
                    next_app_state.set(AppState::InGame);
                    next_game_mode_state.set(GameMode::FreePlay);
                    event_writer.write(StartGameModeEvent(GameMode::FreePlay));
                    info!("fired start game mode event free play");
                }
            }
        }
    }
}
fn handle_game_mode_selection_action_button_press(
    query: Query<
        (&Interaction, &GameModeSelectionActionButton),
        Changed<Interaction>,
    >,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
) {
    for (interaction, game_mode_selection_action_button) in query {
        if let Interaction::Pressed = interaction {
            match game_mode_selection_action_button.0 {
                GameModeSelectionActionButtonType::GoBack => {
                    next_main_menu_state.set(MainMenuState::Root);
                }
            }
        }
    }
}
