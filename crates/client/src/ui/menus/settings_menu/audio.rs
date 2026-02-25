use bevy::{
    prelude::*,
    ui_widgets::{Slider, SliderValue, ValueChange, observe},
};

use crate::{
    game_settings::GameSettings,
    ui::{
        common::DEFAULT_GAME_FONT_PATH,
        menus::settings_menu::SettingsRightSideContentRoot,
        widgets::slider::build_slider,
    },
};

#[derive(Component)]
pub struct VolumeSlider(VolumeSliderType);

enum VolumeSliderType {
    Sounds,
    Music,
}

pub fn spawn_audio_settings_tab_content(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings_right_side: Single<Entity, With<SettingsRightSideContentRoot>>,
    game_settings: Res<GameSettings>,
) {
    info!("Entered CurrentTab::Audio, spawning AudioSettingsTabContent");
    let font_handle = asset_server.load(DEFAULT_GAME_FONT_PATH);

    commands.entity(*settings_right_side).despawn_children();

    commands
        .entity(*settings_right_side)
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Sound Volume"),
                        TextFont {
                            font: font_handle.clone(),
                            ..default()
                        },
                    ));
                    parent.spawn((
                        build_slider(
                            0.0,
                            100.0,
                            game_settings.audio.sounds_volume,
                            VolumeSlider(VolumeSliderType::Sounds),
                        ),
                        observe(update_game_settings_on_volume_slider_change),
                    ));
                });
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Music Volume"),
                        TextFont {
                            font: font_handle,
                            ..default()
                        },
                    ));
                    parent.spawn((
                        build_slider(
                            0.0,
                            100.0,
                            game_settings.audio.music_volume,
                            VolumeSlider(VolumeSliderType::Music),
                        ),
                        observe(update_game_settings_on_volume_slider_change),
                    ));
                });
        });
}

fn update_game_settings_on_volume_slider_change(
    value_change: On<ValueChange<f32>>,
    mut game_settings: ResMut<GameSettings>,
    volume_sliders: Query<&VolumeSlider>,
) {
    let source = value_change.source;
    let Ok(volume_slider) = volume_sliders.get(source) else {
        return;
    };
    match volume_slider.0 {
        VolumeSliderType::Sounds => {
            game_settings.audio.sounds_volume = value_change.value;
        }
        VolumeSliderType::Music => {
            game_settings.audio.music_volume = value_change.value;
        }
    }
}

pub fn update_volume_slider_value(
    game_settings: Res<GameSettings>,
    mut sliders: Query<(Entity, &VolumeSlider), With<Slider>>,
    mut commands: Commands,
) {
    if game_settings.is_changed() {
        for (slider_entity, volume_slider) in sliders.iter_mut() {
            // we insert as component instead of changing the SliderValue component directly,
            // as SliderValue is internally marked as immutable
            match volume_slider.0 {
                VolumeSliderType::Sounds => {
                    commands
                        .entity(slider_entity)
                        .insert(SliderValue(game_settings.audio.sounds_volume));
                }
                VolumeSliderType::Music => {
                    commands
                        .entity(slider_entity)
                        .insert(SliderValue(game_settings.audio.music_volume));
                }
            }
        }
    }
}
