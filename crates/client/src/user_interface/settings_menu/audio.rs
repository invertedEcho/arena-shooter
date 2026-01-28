use bevy::{
    prelude::*,
    ui_widgets::{Slider, SliderValue, ValueChange, observe},
};

use crate::{
    game_settings::GameSettings,
    user_interface::{
        common::DEFAULT_GAME_FONT_PATH, widgets::slider::build_slider,
    },
};

#[derive(Component)]
pub struct VolumeSlider(VolumeSliderType);

enum VolumeSliderType {
    Master,
    Sounds,
    Music,
}

pub fn build_audio_settings_tab_content(
    asset_server: Res<AssetServer>,
    game_settings: Res<GameSettings>,
) -> impl Bundle {
    let game_settings = game_settings.into_inner();
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(8.0),
            ..default()
        },
        children![
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                children![
                    (
                        Text::new("Master Volume"),
                        TextFont {
                            font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                            ..default()
                        }
                    ),
                    (
                        build_slider(
                            0.0,
                            100.0,
                            game_settings.master_volume,
                            VolumeSlider(VolumeSliderType::Master),
                        ),
                        observe(update_game_settings_on_volume_slider_change),
                    )
                ],
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                children![
                    (
                        Text::new("Sound Volume"),
                        TextFont {
                            font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                            ..default()
                        },
                    ),
                    (
                        build_slider(
                            0.0,
                            100.0,
                            game_settings.sounds_volume,
                            VolumeSlider(VolumeSliderType::Sounds),
                        ),
                        observe(update_game_settings_on_volume_slider_change),
                    )
                ],
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                children![
                    (
                        Text::new("Music Volume"),
                        TextFont {
                            font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                            ..default()
                        },
                    ),
                    (
                        build_slider(
                            0.0,
                            100.0,
                            game_settings.music_volume,
                            VolumeSlider(VolumeSliderType::Music),
                        ),
                        observe(update_game_settings_on_volume_slider_change),
                    )
                ],
            )
        ],
    )
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
        VolumeSliderType::Master => {
            game_settings.master_volume = value_change.value;
        }
        VolumeSliderType::Sounds => {
            game_settings.sounds_volume = value_change.value;
        }
        VolumeSliderType::Music => {
            game_settings.music_volume = value_change.value;
        }
    }
}

pub fn update_volume_slider_value(
    res: Res<GameSettings>,
    mut sliders: Query<(Entity, &VolumeSlider), With<Slider>>,
    mut commands: Commands,
) {
    if res.is_changed() {
        for (slider_entity, volume_slider) in sliders.iter_mut() {
            // we insert as component instead of changing the SliderValue component directly,
            // as SliderValue is internally marked as immutable
            match volume_slider.0 {
                VolumeSliderType::Master => {
                    commands
                        .entity(slider_entity)
                        .insert(SliderValue(res.master_volume));
                }
                VolumeSliderType::Sounds => {
                    commands
                        .entity(slider_entity)
                        .insert(SliderValue(res.sounds_volume));
                }
                VolumeSliderType::Music => {
                    commands
                        .entity(slider_entity)
                        .insert(SliderValue(res.music_volume));
                }
            }
        }
    }
}
