use bevy::prelude::*;

use crate::{
    game_flow::GameState,
    user_interface::common::{CommonUiButton, CommonUiButtonType},
};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(Update, handle_main_menu_button_pressed);
    }
}

#[derive(Component)]
struct MainMenuButton(pub MainMenuButtonType);

enum MainMenuButtonType {
    Play,
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
            StateScoped(GameState::MainMenu),
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
                    MainMenuButton(MainMenuButtonType::Play),
                    TextColor::WHITE,
                ))
                .with_child(Text::new("Play"));
            parent
                .spawn((
                    Node { ..default() },
                    Button,
                    CommonUiButton(CommonUiButtonType::Settings),
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
                    CommonUiButton(CommonUiButtonType::Quit),
                    TextColor::WHITE,
                ))
                .with_child(Text::new("Quit"));
        });
}

fn handle_main_menu_button_pressed(
    mut next_game_state: ResMut<NextState<GameState>>,
    query: Query<(&Interaction, &MainMenuButton), Changed<Interaction>>,
) {
    for (interaction, main_menu_button) in query {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match main_menu_button.0 {
            MainMenuButtonType::Play => next_game_state.set(GameState::InGame),
        }
    }
}
