use bevy::{audio::Volume, prelude::*};
use shared::components::DespawnTimer;

use crate::{
    game_flow::states::AppState,
    game_settings::{self, GameSettings},
    player::shooting::messages::PlayerWeaponFiredMessage,
};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_audio_player_container,
                apply_audio_settings.after(spawn_audio_player_container),
                start_main_menu_theme.after(apply_audio_settings),
            ),
        )
        .add_systems(OnEnter(AppState::InGame), stop_music_audio)
        .add_systems(Update, play_shooting_sound_on_player_weapon_fired)
        .add_systems(
            Update,
            update_audio_settings_on_game_settings_change
                .run_if(resource_changed::<GameSettings>),
        );
    }
}

#[derive(Component)]
pub struct MusicAudio;

fn start_main_menu_theme(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    audio_player_container: Single<Entity, With<AudioPlayerContainer>>,
    game_settings: Res<GameSettings>,
) {
    let audio = asset_server.load("music/main_menu_theme.mp3");

    commands.entity(*audio_player_container).with_child((
        AudioPlayer::new(audio),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::Linear(
                (game_settings.music_volume / 100.0).clamp(0.0, 1.0),
            ),
            ..default()
        },
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

fn update_audio_settings_on_game_settings_change(
    game_settings: Res<GameSettings>,
    music_audio_sinks: Query<&mut AudioSink, With<MusicAudio>>,
    mut global_volume: ResMut<GlobalVolume>,
) {
    let master_volume = game_settings.master_volume;
    let new_master_volume =
        Volume::Linear((master_volume / 100.0).clamp(0.0, 1.0));
    global_volume.volume = new_master_volume;

    let music_volume = game_settings.music_volume;
    let new_music_volume =
        Volume::Linear((music_volume / 100.0).clamp(0.0, 1.0));

    for mut music_audio_sink in music_audio_sinks {
        music_audio_sink.set_volume(new_music_volume);
    }
}

fn stop_music_audio(
    mut commands: Commands,
    music_audio_query: Query<Entity, With<MusicAudio>>,
) {
    for music_audio in music_audio_query {
        commands.entity(music_audio).despawn();
    }
}

// We spawn all audioplayers as a child in this component, so we dont get layout shifting in egui
// inspector
#[derive(Component)]
pub struct AudioPlayerContainer;
fn spawn_audio_player_container(mut commands: Commands) {
    commands.spawn((AudioPlayerContainer, Name::new("AudioPlayerContainer")));
}

pub fn play_shooting_sound_on_player_weapon_fired(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut message_reader: MessageReader<PlayerWeaponFiredMessage>,
    audio_player_container: Single<Entity, With<AudioPlayerContainer>>,
) {
    for _ in message_reader.read() {
        let shoot_sound = asset_server.load(
            "sfx/Snake's Authentic Gun Sounds/Full Sound/7.62x39/MP3/762x39 \
             Single MP3.mp3",
        );

        commands.entity(*audio_player_container).with_child((
            AudioPlayer::new(shoot_sound),
            PlaybackSettings::ONCE,
            Name::new("shoot sound player"),
            DespawnTimer(Timer::from_seconds(2.0, TimerMode::Once)),
        ));
    }
}
