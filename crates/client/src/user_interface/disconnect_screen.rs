use bevy::{color::palettes::css::RED, prelude::*};

use crate::{
    game_flow::states::AppState,
    network::GENERIC_NO_CONNECTION_ERROR_MESSAGE,
    user_interface::{
        common::{
            CommonUiButton, DEFAULT_GAME_FONT_PATH, DEFAULT_ROW_GAP,
            UI_BACKGROUND,
        },
        main_menu::{MainMenuCamera, get_main_menu_camera_transform},
        widgets::button::build_common_button,
    },
};

pub struct DisconnectScreenPlugin;

impl Plugin for DisconnectScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Disconnected),
            spawn_disconnected_screen,
        );
    }
}

#[derive(Component)]
struct DisconnectScreenRoot;

fn spawn_disconnected_screen(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    existing_main_menu_camera: Query<&MainMenuCamera>,
) {
    // TODO: optimally this couldnt happen in first place
    if existing_main_menu_camera.count() == 0 {
        commands.spawn((
            MainMenuCamera,
            Camera3d::default(),
            get_main_menu_camera_transform(),
            DespawnOnEnter(AppState::InGame),
        ));
    }

    debug!("Spawning Disconnected screen");
    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: DEFAULT_ROW_GAP,
                ..default()
            },
            BackgroundColor(UI_BACKGROUND),
            DisconnectScreenRoot,
            DespawnOnExit(AppState::Disconnected),
        ))
        .with_children(|parent| {
            parent.spawn(Text::new("Connection lost!"));

            parent.spawn(build_common_button(
                "Return to Main Menu",
                asset_server.load(DEFAULT_GAME_FONT_PATH),
                CommonUiButton::BackToMainMenu,
            ));

            parent
                .spawn((
                    Node {
                        max_width: percent(50.0),
                        ..default()
                    },
                    BorderColor::all(RED),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(GENERIC_NO_CONNECTION_ERROR_MESSAGE),
                        TextLayout {
                            linebreak: LineBreak::WordBoundary,
                            ..default()
                        },
                    ));
                });
        });
}
