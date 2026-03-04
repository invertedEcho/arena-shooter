use bevy::prelude::*;
use shared::{AppRole, CurrentMap, GameModeServer, StartGame};

use crate::{
    game_flow::states::{
        AppState, ClientLoadingState, GameModeClient, MainMenuState,
    },
    ui::{
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
enum MainMenuButton {
    Singleplayer,
    Multiplayer,
    SettingsMainMenu,
    Credits,
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
                "Credits",
                asset_server.load(DEFAULT_GAME_FONT_PATH),
                MainMenuButton::Credits,
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
    mut next_game_mode_state: ResMut<NextState<GameModeClient>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_app_role: ResMut<NextState<AppRole>>,
    mut next_client_loading_state: ResMut<NextState<ClientLoadingState>>,
    mut message_writer: MessageWriter<StartGame>,
) {
    for (interaction, main_menu_button) in main_menu_button_interactions {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match main_menu_button {
            MainMenuButton::Singleplayer => {
                next_main_menu_state.set(MainMenuState::MapSelection);
                next_app_role.set(AppRole::ClientAndServer);
            }
            MainMenuButton::Multiplayer => {
                next_game_mode_state.set(GameModeClient::Multiplayer);
                next_app_state.set(AppState::LoadingGame);
                next_app_role.set(AppRole::ClientOnly);
                // NOTE: we skip state StartingServer, because in multiplayer we dont start a
                // server ourself but connect to the dedicated server
                next_client_loading_state
                    .set(ClientLoadingState::ConnectingToServer);

                info!("Writing StartGame message!");
                message_writer.write(StartGame {
                    game_mode: GameModeServer::FreeForAll,
                    map: CurrentMap::MediumPlastic,
                });
            }
            MainMenuButton::SettingsMainMenu => {
                next_main_menu_state.set(MainMenuState::Settings);
            }
            MainMenuButton::Credits => {
                next_main_menu_state.set(MainMenuState::Credits);
            }
        }
    }
}
