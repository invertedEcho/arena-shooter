use bevy::{audio::Volume, prelude::*};
use shared::components::DespawnTimer;

use crate::{
    game_flow::states::AppState, game_settings::GameSettings,
    player::shooting::messages::PlayerWeaponFiredMessage,
    user_interface::common::AnyButtonInteractionQuery,
};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_audio_player_container,
                start_main_menu_theme.after(spawn_audio_player_container),
            ),
        );
        app.add_systems(OnEnter(AppState::InGame), stop_music_audio);
        app.add_systems(
            Update,
            (play_sound_on_player_weapon_fired, play_button_sound),
        );
        app.add_systems(
            Update,
            update_audio_settings_on_game_settings_change
                .run_if(resource_changed::<GameSettings>),
        );
        app.add_systems(OnEnter(AppState::Disconnected), play_error_sound);
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
            volume: game_settings_volume_to_bevy_volume(
                game_settings.music_volume,
            ),
            ..default()
        },
        MusicAudio,
    ));
}

fn update_audio_settings_on_game_settings_change(
    game_settings: Res<GameSettings>,
    music_audio_sinks: Query<&mut AudioSink, With<MusicAudio>>,
) {
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

pub fn play_sound_on_player_weapon_fired(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut message_reader: MessageReader<PlayerWeaponFiredMessage>,
    audio_player_container: Single<Entity, With<AudioPlayerContainer>>,
    game_settings: Res<GameSettings>,
) {
    for _ in message_reader.read() {
        let shoot_sound = asset_server.load(
            "sfx/Snake's Authentic Gun Sounds/Full Sound/5.56/MP3/556 Single \
             MP3.mp3",
        );

        commands.entity(*audio_player_container).with_child((
            AudioPlayer::new(shoot_sound),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Once,
                volume: game_settings_volume_to_bevy_volume(
                    game_settings.sounds_volume,
                ),
                ..default()
            },
            Name::new("shoot audio player"),
            DespawnTimer(Timer::from_seconds(2.0, TimerMode::Once)),
        ));
    }
}

fn game_settings_volume_to_bevy_volume(game_settings_volume: f32) -> Volume {
    Volume::Linear((game_settings_volume / 100.0).clamp(0.0, 1.0))
}

fn play_button_sound(
    audio_player_container: Single<Entity, With<AudioPlayerContainer>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    query: AnyButtonInteractionQuery,
    game_settings: Res<GameSettings>,
) {
    const HOVER_UI_SFX: &str = "sfx/ui/Hover - 1.ogg";
    const SELECT_UI_SFX: &str = "sfx/ui/Select - 2.ogg";
    for (interaction, _) in query {
        if *interaction == Interaction::None {
            continue;
        }
        let audio_path = if *interaction == Interaction::Hovered {
            HOVER_UI_SFX
        } else {
            SELECT_UI_SFX
        };

        commands.entity(*audio_player_container).with_child((
            AudioPlayer::new(asset_server.load(audio_path)),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Once,
                volume: game_settings_volume_to_bevy_volume(
                    game_settings.sounds_volume,
                ),
                ..default()
            },
            DespawnTimer(Timer::from_seconds(2.0, TimerMode::Once)),
        ));
    }
}

fn play_error_sound(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    game_settings: Res<GameSettings>,
    audio_player_container: Single<Entity, With<AudioPlayerContainer>>,
) {
    const ERROR_UI_SFX: &str = "sfx/ui/Error - 1.ogg";

    commands.entity(*audio_player_container).with_child((
        AudioPlayer::new(asset_server.load(ERROR_UI_SFX)),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Once,
            volume: game_settings_volume_to_bevy_volume(
                game_settings.sounds_volume,
            ),
            ..default()
        },
        Name::new("error audio player"),
        DespawnTimer(Timer::from_seconds(2.0, TimerMode::Once)),
    ));
}
