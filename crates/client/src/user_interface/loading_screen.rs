use bevy::prelude::*;

use crate::{
    game_flow::states::{AppState, LoadingGameState},
    user_interface::shared::build_common_button,
};

pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LoadingGame), spawn_loading_screen);
        app.add_systems(
            Update,
            (
                update_loading_state_text
                    .run_if(state_changed::<LoadingGameState>),
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
                ..default()
            },
            LoadingScreenRoot,
            DespawnOnExit(AppState::LoadingGame),
        ))
        .with_children(|parent| {
            parent.spawn(Text::new("Loading..."));
            parent.spawn((Text::new(""), LoadingStateText));
            parent.spawn((
                build_common_button(asset_server, "Cancel".to_string()),
                LoadingScreenButton::Cancel,
            ));
        });
}

fn update_loading_state_text(
    loading_state: Res<State<LoadingGameState>>,
    mut loading_state_text: Single<&mut Text, With<LoadingStateText>>,
) {
    let loading_state = loading_state.get();
    loading_state_text.0 = loading_state.to_string();
}

fn handle_loading_screen_button_pressed(
    query: Query<(&Interaction, &LoadingScreenButton), Changed<Interaction>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    for (interaction, loading_screen_button) in query {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match loading_screen_button {
            LoadingScreenButton::Cancel => {
                next_app_state.set(AppState::MainMenu);
            }
        }
    }
}
