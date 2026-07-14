use bevy::prelude::*;

use crate::{
    game_flow::states::MainMenuState,
    ui::{
        common::{DEFAULT_GAME_FONT_PATH, DEFAULT_ROW_GAP},
        widgets::button::build_common_button,
    },
};

pub struct ServerSelectionScreenPlugin;

impl Plugin for ServerSelectionScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_server_selection_screen);

        app.add_systems(OnEnter(MainMenuState::ServerSelection), show_screen);
        app.add_systems(OnExit(MainMenuState::ServerSelection), hide_screen);
    }
}

#[derive(Component)]
struct ServerSelectionScreenRoot;

#[derive(Component)]
struct OfficialServerButton;

fn spawn_server_selection_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn((
            ServerSelectionScreenRoot,
            Visibility::Hidden,
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                width: percent(100),
                height: percent(100),
                row_gap: DEFAULT_ROW_GAP,
                ..default()
            },
            Name::new("Server Selection UI Root"),
        ))
        .with_children(|parent| {
            parent.spawn((Text::new("Select a server")));
            parent.spawn(build_common_button(
                "Play on official server",
                asset_server.load(DEFAULT_GAME_FONT_PATH),
                OfficialServerButton,
            ));
            parent.spawn(Text::new("Play on custom server"));
        });
}

fn show_screen(
    visibility: Single<&mut Visibility, With<ServerSelectionScreenRoot>>,
) {
    info!("Making ServerSelectionScreenRoot visible");
    *visibility.into_inner() = Visibility::Visible;
}

fn hide_screen(
    visibility: Single<&mut Visibility, With<ServerSelectionScreenRoot>>,
) {
    info!("Making ServerSelectionScreenRoot hidden");
    *visibility.into_inner() = Visibility::Hidden;
}
