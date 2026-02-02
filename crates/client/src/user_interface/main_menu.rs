use bevy::prelude::*;
use shared::ServerMode;

use crate::{
    game_flow::{
        game_mode::{GameModeClient, StartGameModeMessage},
        states::MainMenuState,
    },
    user_interface::{
        common::{
            CommonUiButton, DEFAULT_GAME_FONT_PATH, DEFAULT_ROW_GAP,
            TITLE_FONT_SIZE, UI_BACKGROUND,
        },
        widgets::button::build_common_button,
    },
};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MainMenuState::Root), spawn_main_menu)
            .add_systems(Update, handle_main_menu_button_pressed);
    }
}

#[derive(Component)]
pub struct MainMenuCamera;

pub fn get_main_menu_camera_transform() -> Transform {
    Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y)
}

#[derive(Component)]
enum MainMenuButton {
    Singleplayer,
    Multiplayer,
    SettingsMainMenu,
}

fn spawn_main_menu(asset_server: Res<AssetServer>, mut commands: Commands) {
    debug!("Spawning Main Menu");
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
            DespawnOnExit(MainMenuState::Root),
            BackgroundColor(UI_BACKGROUND),
            Name::new("Main Menu UI Root"),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    padding: UiRect::new(
                        Val::ZERO,
                        Val::ZERO,
                        Val::ZERO,
                        Val::Px(32.0),
                    ),
                    ..default()
                })
                .with_child((
                    Text::new("Fun Shooter"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: TITLE_FONT_SIZE,
                        ..default()
                    },
                ));
            parent.spawn(build_common_button(
                "Singleplayer",
                asset_server.load(DEFAULT_GAME_FONT_PATH),
                MainMenuButton::Singleplayer,
            ));
            parent.spawn(build_common_button(
                "Multiplayer",
                asset_server.load(DEFAULT_GAME_FONT_PATH),
                MainMenuButton::Multiplayer,
            ));
            parent.spawn(build_common_button(
                "Settings",
                asset_server.load(DEFAULT_GAME_FONT_PATH),
                MainMenuButton::SettingsMainMenu,
            ));
            parent.spawn(build_common_button(
                "Quit",
                asset_server.load(DEFAULT_GAME_FONT_PATH),
                CommonUiButton::Quit,
            ));
        });
}

fn handle_main_menu_button_pressed(
    main_menu_button_interactions: Query<
        (&Interaction, &MainMenuButton),
        Changed<Interaction>,
    >,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
    mut start_game_mode_message_writer: MessageWriter<StartGameModeMessage>,
    mut next_game_mode_state: ResMut<NextState<GameModeClient>>,
    mut server_run_mode: ResMut<NextState<ServerMode>>,
) {
    for (interaction, main_menu_button) in main_menu_button_interactions {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match main_menu_button {
            MainMenuButton::Singleplayer => {
                server_run_mode.set(ServerMode::LocalServerSinglePlayer);
                next_main_menu_state.set(MainMenuState::MapSelection);
            }
            MainMenuButton::Multiplayer => {
                server_run_mode.set(ServerMode::RemoteServer);
                next_game_mode_state.set(GameModeClient::Multiplayer);
                start_game_mode_message_writer
                    .write(StartGameModeMessage { restart: false });
            }
            MainMenuButton::SettingsMainMenu => {
                next_main_menu_state.set(MainMenuState::Settings);
            }
        }
    }
}
