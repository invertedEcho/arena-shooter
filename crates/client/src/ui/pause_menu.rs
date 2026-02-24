use bevy::prelude::*;

use crate::{
    game_flow::states::InGameState,
    ui::common::{CommonUiButton, DEFAULT_FONT_SIZE, DEFAULT_GAME_FONT_PATH},
};

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InGameState::Paused), spawn_pause_menu)
            .add_systems(
                Update,
                (handle_pause_menu_button_pressed)
                    .run_if(in_state(InGameState::Paused)),
            );
    }
}

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub enum PauseMenuButton {
    Resume,
}

fn spawn_pause_menu(asset_server: Res<AssetServer>, mut commands: Commands) {
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
            PauseMenuRoot,
            DespawnOnExit(InGameState::Paused),
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
                    Text::new("Paused"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: DEFAULT_FONT_SIZE,
                        ..default()
                    },
                ));
            parent
                .spawn((
                    Node { ..default() },
                    Button,
                    PauseMenuButton::Resume,
                    TextColor::WHITE,
                ))
                .with_child((
                    Text::new("Resume"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: DEFAULT_FONT_SIZE,
                        ..default()
                    },
                ));
            parent
                .spawn((
                    Node { ..default() },
                    Button,
                    CommonUiButton::BackToMainMenu,
                ))
                .with_child((
                    Text::new("Exit to Main Menu"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: DEFAULT_FONT_SIZE,
                        ..default()
                    },
                ));
        });
}

fn handle_pause_menu_button_pressed(
    mut next_in_game_state: ResMut<NextState<InGameState>>,
    query: Query<(&Interaction, &PauseMenuButton), Changed<Interaction>>,
) {
    for (interaction, pause_menu_button) in query {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match pause_menu_button {
            PauseMenuButton::Resume => {
                next_in_game_state.set(InGameState::Playing);
            }
        }
    }
}
