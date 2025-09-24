use bevy::prelude::*;

use crate::game_flow::AppState;

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start_main_menu_theme)
            .add_systems(OnEnter(AppState::InGame), despawn_main_menu_theme);
    }
}

#[derive(Component)]
pub struct MainMenuTheme;

fn start_main_menu_theme(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let audio = asset_server.load("music/main_menu_theme.mp3");

    commands.spawn((
        AudioPlayer::new(audio),
        PlaybackSettings::LOOP,
        MainMenuTheme,
    ));
}

fn despawn_main_menu_theme(
    mut commands: Commands,
    main_menu_theme_query: Query<Entity, With<MainMenuTheme>>,
) {
    for main_menu_theme in main_menu_theme_query {
        commands.entity(main_menu_theme).despawn();
    }
}
