use bevy::prelude::*;

use crate::{
    game_flow::states::{MainMenuState, SelectedMapState},
    user_interface::{
        DEFAULT_FONT_SIZE, DEFAULT_GAME_FONT_PATH,
        common::{CommonUiButton, CommonUiButtonType},
    },
};

enum MapSelectionButtonType {
    MapSelected(SelectedMapState),
    ContinueToGameModeSelection,
}

#[derive(Component)]
struct MapSelectionButton(MapSelectionButtonType);

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
            update_selected_map_preview_image
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
        ))
        .with_children(|parent| {
            let selected_map_preview_image = match *selected_map_state.get() {
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
                        font_size: DEFAULT_FONT_SIZE,
                        ..default()
                    },
                ));
            parent
                .spawn((
                    Node { ..default() },
                    Button,
                    MapSelectionButton(MapSelectionButtonType::MapSelected(
                        SelectedMapState::TinyTown,
                    )),
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
                    MapSelectionButton(MapSelectionButtonType::MapSelected(
                        SelectedMapState::MediumPlastic,
                    )),
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
                    MapSelectionButton(
                        MapSelectionButtonType::ContinueToGameModeSelection,
                    ),
                    TextColor::WHITE,
                ))
                .with_child((
                    Text::new("Continue to Game Mode Selection"),
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
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
    mut next_selected_map_state: ResMut<NextState<SelectedMapState>>,
) {
    for (interaction, map_selection_button) in query {
        if let Interaction::Pressed = interaction {
            match &map_selection_button.0 {
                MapSelectionButtonType::ContinueToGameModeSelection => {
                    next_main_menu_state.set(MainMenuState::GameModeSelection);
                }
                MapSelectionButtonType::MapSelected(selected_map_state) => {
                    info!("selected map: {:?}", selected_map_state);
                    next_selected_map_state.set(selected_map_state.clone());
                }
            }
        }
    }
}
