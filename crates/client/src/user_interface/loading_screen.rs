use bevy::prelude::*;

use crate::game_flow::states::AppState;

pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LoadingGame), spawn_loading_screen);
    }
}

#[derive(Component)]
struct LoadingScreenRoot;

#[derive(Component)]
struct LoadingStateText;

pub fn spawn_loading_screen(mut commands: Commands) {
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
        });
}
