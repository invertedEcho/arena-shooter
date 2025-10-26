use bevy::prelude::*;

use crate::{
    game_flow::states::{MainMenuState, SelectedMapState},
    user_interface::{
        DEFAULT_FONT_SIZE, DEFAULT_GAME_FONT_PATH,
        common::{CommonUiButton, CommonUiButtonType},
    },
};

#[derive(Component)]
struct MapSelectionButton(pub SelectedMapState);

pub struct MapSelectionPlugin;

impl Plugin for MapSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MainMenuState::MapSelection),
            spawn_map_selection,
        )
        .add_systems(Update, handle_map_selection_button_pressed);
    }
}

fn spawn_map_selection(asset_server: Res<AssetServer>, mut commands: Commands) {
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
            DespawnOnExit(MainMenuState::MapSelection),
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
                    Text::new("Select a Map"),
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
                    MapSelectionButton(SelectedMapState::TinyTown),
                    TextColor::WHITE,
                ))
                .with_child((
                    Text::new("Tiny Town"),
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
                    MapSelectionButton(SelectedMapState::MediumPlastic),
                    TextColor::WHITE,
                ))
                .with_child((
                    Text::new("Medium Plastic"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: DEFAULT_FONT_SIZE,
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
                    CommonUiButton(CommonUiButtonType::BackToMainMenu),
                    TextColor::WHITE,
                ))
                .with_child((
                    Text::new("Go back to Main Menu"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: DEFAULT_FONT_SIZE,
                        ..default()
                    },
                ));
        });
}

fn handle_map_selection_button_pressed(
    query: Query<(&Interaction, &MapSelectionButton), Changed<Interaction>>,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
    mut next_selected_map_state: ResMut<NextState<SelectedMapState>>,
) {
    for (interaction, map_selection_button) in query {
        if let Interaction::Pressed = interaction {
            match map_selection_button.0 {
                SelectedMapState::TinyTown => {
                    next_main_menu_state.set(MainMenuState::GameModeSelection);
                    next_selected_map_state.set(SelectedMapState::TinyTown);
                }
                SelectedMapState::MediumPlastic => {
                    next_main_menu_state.set(MainMenuState::GameModeSelection);
                    next_selected_map_state
                        .set(SelectedMapState::MediumPlastic);
                }
                // TODO: hmm can we really not avoid having these "Nones" in our states?
                SelectedMapState::None => {
                    warn!("This shouldnt happen");
                }
            }
        }
    }
}
