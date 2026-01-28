use bevy::prelude::*;
use shared::{GameModeServer, ServerMode};

use crate::{
    game_flow::{
        game_mode::GameModeClient,
        states::{AppState, MainMenuState},
    },
    user_interface::{
        common::{
            DEFAULT_FONT_SIZE, DEFAULT_GAME_FONT_PATH, DEFAULT_ROW_GAP,
            UI_BACKGROUND,
        },
        widgets::button::build_common_button,
    },
};

pub struct GameModeSelectionUIPlugin;

impl Plugin for GameModeSelectionUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MainMenuState::GameModeSelection),
            spawn_game_mode_selection_screen,
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
struct GameModeSelectionButton(GameModeClient);

#[derive(Component)]
enum GameModeSelectionActionButton {
    GoBack,
}

fn spawn_game_mode_selection_screen(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    info!("Spawning GameModeSelectionScreen");
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: DEFAULT_ROW_GAP,
                ..default()
            },
            GameModeSelectionScreen,
            DespawnOnExit(MainMenuState::GameModeSelection),
            BackgroundColor(UI_BACKGROUND),
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
                .with_child((
                    Text::new("Select a game mode"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: DEFAULT_FONT_SIZE,
                        ..default()
                    },
                ));
            parent.spawn((
                build_common_button(
                    "Waves",
                    asset_server.load(DEFAULT_GAME_FONT_PATH),
                ),
                GameModeSelectionButton(GameModeClient::Waves),
            ));
            parent.spawn((
                build_common_button(
                    "Free Roam",
                    asset_server.load(DEFAULT_GAME_FONT_PATH),
                ),
                GameModeSelectionButton(GameModeClient::FreeRoam),
            ));
            parent.spawn((
                build_common_button(
                    "Go back",
                    asset_server.load(DEFAULT_GAME_FONT_PATH),
                ),
                GameModeSelectionActionButton::GoBack,
            ));
        });
}

fn handle_game_mode_selection_button_press(
    query: Query<
        (&Interaction, &GameModeSelectionButton),
        Changed<Interaction>,
    >,
    mut next_game_mode_state: ResMut<NextState<GameModeClient>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut game_mode_server: Query<&mut GameModeServer>,
    server_mode: Res<State<ServerMode>>,
) {
    for (interaction, game_mode_selection_button) in query {
        if let Interaction::Pressed = interaction {
            let pressed_game_mode = game_mode_selection_button.0;
            next_game_mode_state.set(pressed_game_mode);
            next_app_state.set(AppState::LoadingGame);

            if *server_mode.get() == ServerMode::LocalServerSinglePlayer
                && let Ok(mut game_mode_server) = game_mode_server.single_mut()
            {
                match pressed_game_mode {
                    GameModeClient::FreeRoam => {
                        *game_mode_server = GameModeServer::FreeForAll;
                    }
                    GameModeClient::Waves => {
                        *game_mode_server = GameModeServer::Waves;
                    }
                    _ => {}
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
            match game_mode_selection_action_button {
                GameModeSelectionActionButton::GoBack => {
                    next_main_menu_state.set(MainMenuState::MapSelection);
                }
            }
        }
    }
}
