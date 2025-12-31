use bevy::{audio::Volume, prelude::*};

use crate::{
    game_flow::states::AppState, game_settings::GameSettings,
    player::shooting::messages::PlayerWeaponFiredMessage,
    shared::components::DespawnTimer,
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
        .add_systems(Update, play_shooting_sound_on_player_weapon_fired);
    }
}

#[derive(Component)]
pub struct MusicAudio;

fn start_main_menu_theme(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    audio_player_container: Single<Entity, With<AudioPlayerContainer>>,
) {
    let audio = asset_server.load("music/main_menu_theme.mp3");

    commands.entity(*audio_player_container).with_child((
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

// We spawn all audioplayers as a child in this component, so we dont get layout shifting in egui
// inspector
#[derive(Component)]
pub struct AudioPlayerContainer;
fn spawn_audio_player_container(mut commands: Commands) {
    commands.spawn(AudioPlayerContainer);
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
