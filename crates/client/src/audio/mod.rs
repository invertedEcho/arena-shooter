use bevy::{audio::Volume, prelude::*};
use lightyear::prelude::Controlled;
use rand::seq::IndexedRandom;
use shared::{
    character_controller::components::Grounded,
    components::DespawnTimer,
    player::{AimType, PlayerState},
    shooting::{PlayerWeapons, WeaponKind},
};

use crate::{
    game_flow::states::AppState,
    game_settings::GameSettings,
    player::shooting::messages::{
        PlayerWeaponFiredMessage, PlayerWeaponSlotChangeMessage,
        ReloadPlayerWeaponMessage,
    },
    ui::common::AnyButtonInteractionQuery,
};

const BASE_PATH_TO_ASSAULT_RIFLE_SOUNDS: &str = "sfx/weapons/assault_rifle/";
const BASE_PATH_TO_PISTOL_SOUNDS: &str = "sfx/weapons/pistol_walther_p38/";
const BASE_PATH_TO_WEAPON_SOUNDS: &str = "sfx/weapons/";

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySoundMessage>();
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
            (
                handle_play_sound_message,
                play_sound_on_player_weapon_fired,
                play_button_sound,
                play_footstep_sound,
                play_weapon_slot_change_audio,
                play_aim_sound_on_changed_aim_type,
                play_reload_sound,
            ),
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
                game_settings.audio.music_volume,
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
    let music_volume = game_settings.audio.music_volume;
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
    mut message_reader: MessageReader<PlayerWeaponFiredMessage>,
    player_query: Single<(&PlayerWeapons, &PlayerState)>,
    mut play_sound_message_writer: MessageWriter<PlaySoundMessage>,
) {
    let (player_weapons, player_state) = player_query.into_inner();
    for _ in message_reader.read() {
        let current_weapon =
            &player_weapons.weapons[player_state.active_weapon_slot];

        let shoot_sound = match current_weapon.game_weapon.kind {
            WeaponKind::AK47 => {
                &(BASE_PATH_TO_ASSAULT_RIFLE_SOUNDS.to_string() + "shoot.mp3")
            }
            WeaponKind::Glock => {
                &(BASE_PATH_TO_PISTOL_SOUNDS.to_string() + "shoot.ogg")
            }
            // TODO: hmm we need more weapon sounds
            WeaponKind::P90 => {
                &(BASE_PATH_TO_WEAPON_SOUNDS.to_string() + "smg/shoot.mp3")
            }
        };
        play_sound_message_writer.write(PlaySoundMessage {
            path_to_audio: shoot_sound.to_string(),
        });
    }
}

fn game_settings_volume_to_bevy_volume(game_settings_volume: f32) -> Volume {
    Volume::Linear((game_settings_volume / 100.0).clamp(0.0, 1.0))
}

fn play_button_sound(
    query: AnyButtonInteractionQuery,
    mut play_sound_message_writer: MessageWriter<PlaySoundMessage>,
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

        play_sound_message_writer.write(PlaySoundMessage {
            path_to_audio: audio_path.to_string(),
        });
    }
}

fn play_error_sound(
    mut play_sound_message_writer: MessageWriter<PlaySoundMessage>,
) {
    const ERROR_UI_SFX: &str = "sfx/ui/Error - 1.ogg";
    play_sound_message_writer.write(PlaySoundMessage {
        path_to_audio: ERROR_UI_SFX.to_string(),
    });
}

fn play_footstep_sound(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut last_play: Local<f32>,
    time: Res<Time>,
    grounded: Single<&Grounded, With<Controlled>>,
    mut play_sound_message_writer: MessageWriter<PlaySoundMessage>,
) {
    *last_play += time.delta_secs();
    if *last_play < 0.3 {
        return;
    }

    const ALL_FOOTSTEP_SOUNDS: [&str; 4] = [
        "sfx/footsteps/FootstepsConcrete1.ogg",
        "sfx/footsteps/FootstepsConcrete2.ogg",
        "sfx/footsteps/FootstepsConcrete3.ogg",
        "sfx/footsteps/FootstepsConcrete4.ogg",
    ];
    if (keyboard_input.pressed(KeyCode::KeyW)
        || keyboard_input.pressed(KeyCode::KeyA)
        || keyboard_input.pressed(KeyCode::KeyS)
        || keyboard_input.pressed(KeyCode::KeyD))
        && grounded.0
    {
        let mut rng = rand::rng();
        let footstep_sound = ALL_FOOTSTEP_SOUNDS.choose(&mut rng);

        if let Some(footstep_sound) = footstep_sound {
            play_sound_message_writer.write(PlaySoundMessage {
                path_to_audio: footstep_sound.to_string(),
            });
            *last_play = 0.0;
        }
    }
}

#[derive(Message)]
pub struct PlaySoundMessage {
    path_to_audio: String,
}

fn handle_play_sound_message(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    game_settings: Res<GameSettings>,
    audio_player_container: Single<Entity, With<AudioPlayerContainer>>,
    mut message_reader: MessageReader<PlaySoundMessage>,
) {
    for message in message_reader.read() {
        commands.entity(*audio_player_container).with_child((
            AudioPlayer::new(asset_server.load(message.path_to_audio.clone())),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Once,
                volume: game_settings_volume_to_bevy_volume(
                    game_settings.audio.sounds_volume,
                ),
                ..default()
            },
            DespawnTimer(Timer::from_seconds(5.0, TimerMode::Once)),
        ));
    }
}

fn play_weapon_slot_change_audio(
    mut message_reader: MessageReader<PlayerWeaponSlotChangeMessage>,
    mut play_sound_message_writer: MessageWriter<PlaySoundMessage>,
    player_weapons: Single<&PlayerWeapons>,
) {
    for message in message_reader.read() {
        let new_weapon = &player_weapons.weapons[message.0];
        let path = match new_weapon.game_weapon.kind {
            WeaponKind::Glock => {
                BASE_PATH_TO_PISTOL_SOUNDS.to_string() + "equip.ogg"
            }
            WeaponKind::AK47 | WeaponKind::P90 => {
                BASE_PATH_TO_ASSAULT_RIFLE_SOUNDS.to_string() + "equip.mp3"
            }
        };

        play_sound_message_writer.write(PlaySoundMessage {
            path_to_audio: path,
        });
    }
}

fn play_aim_sound_on_changed_aim_type(
    mut play_sound_message_writer: MessageWriter<PlaySoundMessage>,
    new_aim_type: Single<&AimType, Changed<AimType>>,
    player_query: Single<(&PlayerWeapons, &PlayerState)>,
) {
    let (player_weapons, player_state) = player_query.into_inner();

    if **new_aim_type != AimType::Scoped {
        return;
    }

    let current_weapon =
        &player_weapons.weapons[player_state.active_weapon_slot];
    let current_weapon_stats = &current_weapon.game_weapon;

    if current_weapon_stats.kind == WeaponKind::Glock {
        play_sound_message_writer.write(PlaySoundMessage {
            path_to_audio: BASE_PATH_TO_PISTOL_SOUNDS.to_string() + "aim.ogg",
        });
    }
}

fn play_reload_sound(
    mut play_sound_message_writer: MessageWriter<PlaySoundMessage>,
    mut message_reader: MessageReader<ReloadPlayerWeaponMessage>,
    player_query: Single<(&PlayerWeapons, &PlayerState)>,
) {
    let (player_weapons, player_state) = player_query.into_inner();
    for _ in message_reader.read() {
        let current_weapon_kind = &player_weapons.weapons
            [player_state.active_weapon_slot]
            .game_weapon
            .kind;

        let path = match current_weapon_kind {
            WeaponKind::Glock => {
                BASE_PATH_TO_PISTOL_SOUNDS.to_string() + "reload.ogg"
            }
            WeaponKind::AK47 | WeaponKind::P90 => {
                BASE_PATH_TO_ASSAULT_RIFLE_SOUNDS.to_string() + "reload.mp3"
            }
        };
        play_sound_message_writer.write(PlaySoundMessage {
            path_to_audio: path,
        });
    }
}
