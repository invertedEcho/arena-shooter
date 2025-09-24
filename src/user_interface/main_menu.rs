use bevy::prelude::*;

use crate::game_flow::AppState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), spawn_main_menu)
            .add_systems(Update, handle_main_menu_button_pressed);
    }
}

#[derive(Component)]
struct MainMenuButton(pub MainMenuButtonType);

enum MainMenuButtonType {
    Singleplayer,
    SettingsMainMenu,
    Quit,
}

fn spawn_main_menu(mut commands: Commands) {
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
            StateScoped(AppState::MainMenu),
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
                .with_child(Text::new("Main Menu"));
            parent
                .spawn((
                    Node { ..default() },
                    Button,
                    MainMenuButton(MainMenuButtonType::Singleplayer),
                    TextColor::WHITE,
                ))
                .with_child(Text::new("Singleplayer"));
            parent
                .spawn((
                    Node { ..default() },
                    Button,
                    MainMenuButton(MainMenuButtonType::SettingsMainMenu),
                ))
                .with_child(Text::new("Settings"));
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
                .with_child(Text::new("Quit"));
        });
}

fn handle_main_menu_button_pressed(
    main_menu_button_interactions: Query<
        (&Interaction, &MainMenuButton),
        Changed<Interaction>,
    >,
    mut next_game_state: ResMut<NextState<AppState>>,
    mut app_exit_event_writer: EventWriter<AppExit>,
) {
    for (interaction, main_menu_button) in main_menu_button_interactions {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match main_menu_button.0 {
            MainMenuButtonType::Singleplayer => {
                next_game_state.set(AppState::GameModeSelection)
            }
            MainMenuButtonType::SettingsMainMenu => {
                next_game_state.set(AppState::SettingsMainMenu)
            }
            MainMenuButtonType::Quit => {
                app_exit_event_writer.write(AppExit::Success);
            }
        }
    }
}
