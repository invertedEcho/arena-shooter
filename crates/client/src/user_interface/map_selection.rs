use bevy::prelude::*;
use shared::SelectedMapState;

use crate::{
    game_flow::states::MainMenuState,
    user_interface::{
        common::CommonUiButton,
        shared::{
            DEFAULT_GAME_FONT_PATH, NORMAL_FONT_SIZE, UI_BACKGROUND,
            UI_SELECTED, UI_TEXT,
        },
    },
};

#[derive(Component)]
pub struct MapSelectionButton(SelectedMapState);

#[derive(Component)]
struct SelectedMapPreviewImage;

pub struct MapSelectionPlugin;

impl Plugin for MapSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MainMenuState::MapSelection),
            spawn_map_selection,
        )
        .add_systems(
            Update,
            (
                update_selected_map_preview_image,
                update_selected_map_button_color,
            )
                .run_if(state_changed::<SelectedMapState>),
        )
        .add_systems(Update, handle_map_selection_button_pressed);
    }
}

fn spawn_map_selection(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    selected_map_state: Res<State<SelectedMapState>>,
) {
    let selected_map_state = selected_map_state.get();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_grow: 1.0,
                ..default()
            },
            DespawnOnExit(MainMenuState::MapSelection),
            BackgroundColor(UI_BACKGROUND),
        ))
        .with_children(|parent| {
            let selected_map_preview_image = match selected_map_state {
                SelectedMapState::TinyTown => "maps/tiny_town/preview.png",
                SelectedMapState::MediumPlastic => {
                    "maps/medium_plastic/preview.png"
                }
            };
            parent
                .spawn(Node {
                    width: Val::Percent(30.0),
                    height: Val::Auto,
                    padding: UiRect::new(
                        Val::ZERO,
                        Val::ZERO,
                        Val::ZERO,
                        Val::Px(16.0),
                    ),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|image_container| {
                    image_container.spawn((
                        SelectedMapPreviewImage,
                        ImageNode {
                            image: asset_server
                                .load(selected_map_preview_image),
                            ..default()
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Auto,
                            aspect_ratio: Some(16.0 / 9.0),
                            ..default()
                        },
                    ));
                });
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
                        font_size: NORMAL_FONT_SIZE,
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
                        font_size: NORMAL_FONT_SIZE,
                        ..default()
                    },
                    TextColor(get_text_button_color_for_map_selection_button(
                        selected_map_state,
                        MapSelectionButton(SelectedMapState::TinyTown),
                    )),
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
                        font_size: NORMAL_FONT_SIZE,
                        ..default()
                    },
                    TextColor(get_text_button_color_for_map_selection_button(
                        selected_map_state,
                        MapSelectionButton(SelectedMapState::MediumPlastic),
                    )),
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
                    CommonUiButton::ToGameModeSelection,
                    TextColor::WHITE,
                ))
                .with_child((
                    Text::new("Continue to Game Mode Selection"),
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
                    CommonUiButton::BackToMainMenu,
                    TextColor::WHITE,
                ))
                .with_child((
                    Text::new("Go back to Main Menu"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: NORMAL_FONT_SIZE,
                        ..default()
                    },
                ));
        });
}

fn get_text_button_color_for_map_selection_button(
    selected_map_state: &SelectedMapState,
    button: MapSelectionButton,
) -> Color {
    if button.0 == *selected_map_state {
        UI_SELECTED
    } else {
        UI_TEXT
    }
}

fn update_selected_map_preview_image(
    asset_server: Res<AssetServer>,
    mut image_node: Single<&mut ImageNode, With<SelectedMapPreviewImage>>,
    selected_map_state: Res<State<SelectedMapState>>,
) {
    let selected_map_preview_image = match *selected_map_state.get() {
        SelectedMapState::TinyTown => "maps/tiny_town/preview.png",
        SelectedMapState::MediumPlastic => "maps/medium_plastic/preview.png",
    };
    image_node.image = asset_server.load(selected_map_preview_image);
}

fn handle_map_selection_button_pressed(
    query: Query<(&Interaction, &MapSelectionButton), Changed<Interaction>>,
    mut next_selected_map_state: ResMut<NextState<SelectedMapState>>,
) {
    for (interaction, map_selection_button) in query {
        if let Interaction::Pressed = interaction {
            next_selected_map_state.set(map_selection_button.0.clone());
        }
    }
}

fn update_selected_map_button_color(
    query: Query<(&MapSelectionButton, &Children)>,
    selected_map_state: Res<State<SelectedMapState>>,
    mut text_color_query: Query<&mut TextColor>,
) {
    let selected_map_state = selected_map_state.get();

    for (map_selection_button, children) in query {
        let Ok(mut text_color) = text_color_query.get_mut(children[0]) else {
            continue;
        };
        if map_selection_button.0 == *selected_map_state {
            **text_color = UI_SELECTED;
        } else {
            **text_color = UI_TEXT;
        }
    }
}
