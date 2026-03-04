use bevy::prelude::*;
use shared::StopGame;

use crate::{
    game_flow::states::{AppState, ClientLoadingState},
    ui::{
        common::{DEFAULT_GAME_FONT_PATH, DEFAULT_ROW_GAP},
        widgets::button::build_common_button,
    },
};

pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LoadingGame), spawn_loading_screen);
        app.add_systems(
            Update,
            (
                update_loading_state_text
                    .run_if(state_changed::<ClientLoadingState>),
                handle_loading_screen_button_pressed,
            ),
        );
    }
}

#[derive(Component)]
struct LoadingScreenRoot;

#[derive(Component)]
struct LoadingStateText;

#[derive(Component)]
enum LoadingScreenButton {
    Cancel,
}

pub fn spawn_loading_screen(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    info!("Spawning Loading screen");

    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: DEFAULT_ROW_GAP,
                ..default()
            },
            LoadingScreenRoot,
            DespawnOnExit(AppState::LoadingGame),
        ))
        .with_children(|parent| {
            parent.spawn(Text::new("Loading..."));
            parent.spawn((Text::new(""), LoadingStateText));
            parent.spawn(build_common_button(
                "Cancel",
                asset_server.load(DEFAULT_GAME_FONT_PATH),
                LoadingScreenButton::Cancel,
            ));
        });
}

fn update_loading_state_text(
    loading_state: Res<State<ClientLoadingState>>,
    mut loading_state_text: Single<&mut Text, With<LoadingStateText>>,
) {
    let loading_state = loading_state.get();
    loading_state_text.0 = loading_state.to_string();
}

fn handle_loading_screen_button_pressed(
    interaction_query: Query<
        (&Interaction, &LoadingScreenButton),
        Changed<Interaction>,
    >,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut stop_game_message_writer: MessageWriter<StopGame>,
) {
    for (interaction, loading_screen_button) in interaction_query {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match loading_screen_button {
            LoadingScreenButton::Cancel => {
                next_app_state.set(AppState::MainMenu);
                stop_game_message_writer.write(StopGame);
            }
        }
    }
}
