use bevy::{audio::Volume, prelude::*};

use crate::{game_flow::states::AppState, game_settings::GameSettings};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                apply_audio_settings,
                start_main_menu_theme.after(apply_audio_settings),
            ),
        )
        .add_systems(OnEnter(AppState::InGame), stop_music_audio);
    }
}

#[derive(Component)]
pub struct MusicAudio;

fn start_main_menu_theme(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let audio = asset_server.load("music/main_menu_theme.mp3");

    commands.spawn((
        AudioPlayer::new(audio),
        PlaybackSettings::LOOP,
        MusicAudio,
    ));
}

fn apply_audio_settings(
    game_settings: Res<GameSettings>,
    mut global_volume: ResMut<GlobalVolume>,
) {
    let volume = Volume::Linear(game_settings.master_volume / 100.0);
    global_volume.volume = volume;
}

fn stop_music_audio(
    mut commands: Commands,
    music_audio_query: Query<Entity, With<MusicAudio>>,
) {
    for music_audio in music_audio_query {
        commands.entity(music_audio).despawn();
    }
}
