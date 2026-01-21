use bevy::prelude::*;

use crate::{
    game_flow::{
        game_mode::{GameModeState, StartGameModeMessage},
        states::MainMenuState,
    },
    user_interface::shared::{
        DEFAULT_GAME_FONT_PATH, NORMAL_FONT_SIZE, TITLE_FONT_SIZE, UI_BG,
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
struct MainMenuButton(pub MainMenuButtonType);

enum MainMenuButtonType {
    Singleplayer,
    Multiplayer,
    SettingsMainMenu,
    Quit,
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
                ..default()
            },
            DespawnOnExit(MainMenuState::Root),
            BackgroundColor(UI_BG),
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
            parent
                .spawn((
                    Node { ..default() },
                    Button,
                    MainMenuButton(MainMenuButtonType::Singleplayer),
                    TextColor::WHITE,
                ))
                .with_child((
                    Text::new("Singleplayer"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: NORMAL_FONT_SIZE,
                        ..default()
                    },
                ));
            parent
                .spawn((
                    Node { ..default() },
                    Button,
                    MainMenuButton(MainMenuButtonType::Multiplayer),
                    TextColor::WHITE,
                ))
                .with_child((
                    Text::new("Multiplayer (FFA)"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: NORMAL_FONT_SIZE,
                        ..default()
                    },
                ));
            parent
                .spawn((
                    Node { ..default() },
                    Button,
                    MainMenuButton(MainMenuButtonType::SettingsMainMenu),
                ))
                .with_child((
                    Text::new("Settings"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: NORMAL_FONT_SIZE,
                        ..default()
                    },
                ));
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
                    MainMenuButton(MainMenuButtonType::Quit),
                    TextColor::WHITE,
                ))
                .with_child((
                    Text::new("Quit"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: NORMAL_FONT_SIZE,
                        ..default()
                    },
                ));
        });
}

fn handle_main_menu_button_pressed(
    main_menu_button_interactions: Query<
        (&Interaction, &MainMenuButton),
        Changed<Interaction>,
    >,
    mut app_exit_message_writer: MessageWriter<AppExit>,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
    mut start_game_mode_message_writer: MessageWriter<StartGameModeMessage>,
    mut next_game_mode_state: ResMut<NextState<GameModeState>>,
) {
    for (interaction, main_menu_button) in main_menu_button_interactions {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match main_menu_button.0 {
            MainMenuButtonType::Singleplayer => {
                next_main_menu_state.set(MainMenuState::MapSelection);
            }
            MainMenuButtonType::Multiplayer => {
                next_game_mode_state.set(GameModeState::Multiplayer);
                start_game_mode_message_writer
                    .write(StartGameModeMessage { restart: false });
            }
            MainMenuButtonType::SettingsMainMenu => {
                next_main_menu_state.set(MainMenuState::Settings);
            }
            MainMenuButtonType::Quit => {
                app_exit_message_writer.write(AppExit::Success);
            }
        }
    }
}
