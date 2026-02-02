use bevy::{color::palettes::css::RED, prelude::*};

use crate::{
    game_flow::states::{DisconnectedState, InGameState},
    user_interface::{
        common::{CommonUiButton, DEFAULT_GAME_FONT_PATH},
        main_menu::{MainMenuCamera, get_main_menu_camera_transform},
        widgets::button::build_common_button,
    },
};

pub struct DisconnectScreenPlugin;

impl Plugin for DisconnectScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::Disconnected),
            spawn_disconnected_screen,
        );
    }
}

#[derive(Component)]
struct DisconnectScreenRoot;

fn spawn_disconnected_screen(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    disconnect_state: Res<State<DisconnectedState>>,
    existing_main_menu_camera: Query<&MainMenuCamera>,
) {
    // TODO: optimally this couldnt happen in first place
    if existing_main_menu_camera.count() == 0 {
        commands.spawn((
            MainMenuCamera,
            Camera3d::default(),
            get_main_menu_camera_transform(),
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
                ..default()
            },
            DisconnectScreenRoot,
            DespawnOnExit(InGameState::Disconnected),
        ))
        .with_children(|parent| {
            parent.spawn(Text::new("CONNECTION LOST"));

            parent.spawn(Text::new("Attempting to reconnect..."));

            parent.spawn(build_common_button(
                "Return to Main Menu",
                asset_server.load(DEFAULT_GAME_FONT_PATH),
                CommonUiButton::BackToMainMenu,
            ));

            parent
                .spawn((
                    Node {
                        align_self: AlignSelf::End,
                        justify_self: JustifySelf::End,
                        justify_content: JustifyContent::End,
                        ..default()
                    },
                    BorderColor::all(RED),
                ))
                .with_children(|parent| match disconnect_state.get() {
                    DisconnectedState::Reason(reason) => {
                        parent.spawn(Text::new("Reason: "));
                        parent.spawn(Text::new(reason));
                    }
                    DisconnectedState::Reconnecting => {
                        parent.spawn(Text::new("Reconnecting..."));
                    }
                });
        });
}
