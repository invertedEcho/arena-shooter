use bevy::{image::ToExtents, prelude::*};

use crate::{
    game_flow::states::MainMenuState,
    user_interface::{
        DEFAULT_GAME_FONT_PATH, SUB_HEADER_FONT_SIZE, TITLE_FONT_SIZE,
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

#[derive(Component)]
struct MainMenuButton(pub MainMenuButtonType);

enum MainMenuButtonType {
    Singleplayer,
    Multiplayer,
    SettingsMainMenu,
    Quit,
}

fn spawn_main_menu(asset_server: Res<AssetServer>, mut commands: Commands) {
    info!("Spawning Main Menu");
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
                        font_size: SUB_HEADER_FONT_SIZE,
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
                    Text::new("Multiplayer"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: SUB_HEADER_FONT_SIZE,
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
                        font_size: SUB_HEADER_FONT_SIZE,
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
                        font_size: SUB_HEADER_FONT_SIZE,
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
                // connect to server and spawn map
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
