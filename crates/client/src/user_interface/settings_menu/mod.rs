use crate::{
    audio::MusicAudio,
    game_flow::states::MainMenuState,
    game_settings::{GameSettings, update_game_settings_file},
    user_interface::{
        settings_menu::{
            audio::{
                build_audio_settings_tab_content, update_volume_slider_value,
            },
            graphics::{GraphicsCheckbox, GraphicsCheckboxType},
        },
        shared::{
            DEFAULT_GAME_FONT_PATH, UI_BORDER, UI_PRIMARY, UI_SELECTED, UI_TEXT,
        },
        widgets::checkbox::build_checkbox,
    },
};
use bevy::{
    audio::Volume,
    color::palettes::tailwind::SLATE_600,
    prelude::*,
    ui::Checked,
    ui_widgets::{ValueChange, observe},
    window::WindowMode,
};

mod audio;
mod graphics;

pub struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SelectedTabSettings>()
            .add_message::<ApplyGameSettingsMessage>()
            .add_systems(OnEnter(MainMenuState::Settings), spawn_settings_menu)
            .add_systems(
                Update,
                (
                    handle_settings_tab_changed,
                    update_settings_tab_button_color,
                )
                    .run_if(state_changed::<SelectedTabSettings>),
            )
            .add_systems(
                Update,
                (
                    handle_settings_menu_button_pressed,
                    update_volume_slider_value,
                    apply_game_settings,
                    handle_change_tab_button_pressed,
                ),
            );
    }
}

#[derive(SubStates, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[source(MainMenuState = MainMenuState::Settings)]
pub enum SelectedTabSettings {
    #[default]
    Audio,
    Graphics,
    Controls,
}

// TODO: no need for extra struct, components can also be enums
#[derive(Component)]
struct SettingsMenuButton(pub SettingsButtonType);

#[derive(Component)]
pub struct SettingsChangeTabButton(pub SelectedTabSettings);

enum SettingsButtonType {
    ToggleFullscreen,
    Back,
    Apply,
}

#[derive(Component)]
struct SettingsMenuRoot;

#[derive(Component)]
struct SettingsRightSideContentRoot;

fn spawn_settings_menu(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    game_settings: Res<GameSettings>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(75.0),
                height: Val::Percent(75.0),
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                column_gap: px(16.0),
                border: UiRect::all(px(2.0)),
                ..default()
            },
            SettingsMenuRoot,
            Name::new("Settings Menu Root Node"),
            DespawnOnExit(MainMenuState::Settings),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("LeftSide"),
                    Node {
                        width: percent(25.0),
                        height: percent(100.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::End,
                            row_gap: px(16.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    padding: UiRect::all(px(8)),
                                    width: percent(50),
                                    justify_content: JustifyContent::Center,
                                    border: UiRect::left(px(4)),
                                    ..default()
                                },
                                BorderColor::all(UI_BORDER),
                                BackgroundColor(UI_PRIMARY),
                                Button,
                                SettingsChangeTabButton(
                                    SelectedTabSettings::Audio,
                                ),
                                children![(
                                    Text::new("Audio"),
                                    TextFont {
                                        font: asset_server
                                            .load(DEFAULT_GAME_FONT_PATH),
                                        ..default()
                                    },
                                )],
                            ));
                            parent.spawn((
                                Node {
                                    padding: UiRect::all(px(8)),
                                    width: percent(50),
                                    justify_content: JustifyContent::Center,
                                    border: UiRect::left(px(4)),
                                    ..default()
                                },
                                BorderColor::all(UI_SELECTED),
                                Button,
                                BackgroundColor(UI_PRIMARY),
                                SettingsChangeTabButton(
                                    SelectedTabSettings::Graphics,
                                ),
                                children![(
                                    Text::new("Graphics"),
                                    TextFont {
                                        font: asset_server
                                            .load(DEFAULT_GAME_FONT_PATH),
                                        ..default()
                                    },
                                )],
                            ));
                            parent.spawn((
                                Node {
                                    padding: UiRect::all(px(8)),
                                    width: percent(50),
                                    justify_content: JustifyContent::Center,
                                    border: UiRect::left(px(4)),
                                    ..default()
                                },
                                BorderColor::all(UI_SELECTED),
                                Button,
                                BackgroundColor(UI_PRIMARY),
                                SettingsChangeTabButton(
                                    SelectedTabSettings::Controls,
                                ),
                                children![(
                                    Text::new("Controls"),
                                    TextFont {
                                        font: asset_server
                                            .load(DEFAULT_GAME_FONT_PATH),
                                        ..default()
                                    }
                                )],
                            ));
                        });
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::End,
                            row_gap: px(16.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    padding: UiRect::all(px(8)),
                                    width: percent(50),
                                    justify_content: JustifyContent::Center,
                                    border: UiRect::left(px(4.0)),
                                    ..default()
                                },
                                Button,
                                BackgroundColor(UI_PRIMARY),
                                BorderColor::all(UI_BORDER),
                                SettingsMenuButton(SettingsButtonType::Apply),
                                children![(
                                    Text::new("Apply"),
                                    TextFont {
                                        font: asset_server
                                            .load(DEFAULT_GAME_FONT_PATH),
                                        ..default()
                                    },
                                    TextColor(UI_TEXT)
                                )],
                            ));
                            parent.spawn((
                                Node {
                                    padding: UiRect::all(px(8)),
                                    width: percent(50),
                                    justify_content: JustifyContent::Center,
                                    border: UiRect::left(px(4.0)),
                                    ..default()
                                },
                                Button,
                                BackgroundColor(UI_PRIMARY),
                                BorderColor::all(UI_BORDER),
                                SettingsMenuButton(SettingsButtonType::Back),
                                children![(
                                    Text::new("Back"),
                                    TextFont {
                                        font: asset_server
                                            .load(DEFAULT_GAME_FONT_PATH),
                                        ..default()
                                    },
                                    TextColor(UI_TEXT)
                                )],
                            ));
                        });
                });
            parent
                .spawn((
                    Name::new("RightSideRoot"),
                    Node {
                        width: percent(75.0),
                        height: percent(100.0),
                        ..default()
                    },
                    BackgroundColor(SLATE_600.into()),
                ))
                .with_child((
                    Node {
                        width: percent(100.0),
                        height: percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8.0),
                        padding: UiRect::all(px(8.0)),
                        ..default()
                    },
                    Name::new("RightSideContentRoot"),
                    SettingsRightSideContentRoot,
                    children![build_audio_settings_tab_content(
                        asset_server,
                        game_settings,
                    )],
                ));
        });
}

// We sadly cant use a function which gets the required components depending on the selected tab,
// as it causes issues with arms having different types even if they are all of type Bundle
fn handle_settings_tab_changed(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    settings_right_side: Single<Entity, With<SettingsRightSideContentRoot>>,
    settings_tab_state: Res<State<SelectedTabSettings>>,
    game_settings: Res<GameSettings>,
) {
    commands.entity(*settings_right_side).despawn_children();
    let settings_tab_state = settings_tab_state.get();
    match settings_tab_state {
        SelectedTabSettings::Audio => {
            commands.entity(*settings_right_side).with_child(
                build_audio_settings_tab_content(asset_server, game_settings),
            );
        }
        SelectedTabSettings::Graphics => {
            commands
                .entity(*settings_right_side)
                .with_children(|parent| {
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },))
                        .with_children(|parent| {
                            parent.spawn((
                                Button,
                                SettingsMenuButton(
                                    SettingsButtonType::ToggleFullscreen,
                                ),
                                children![
                                    Text::new("Borderless Fullscreen"),
                                    TextFont {
                                        font: asset_server
                                            .load(DEFAULT_GAME_FONT_PATH),
                                        ..default()
                                    },
                                ],
                            ));
                            parent
                                .spawn((
                                    build_checkbox(
                                        &asset_server,
                                        "",
                                        GraphicsCheckbox(
                                            GraphicsCheckboxType::Fullscreen,
                                        ),
                                    ),
                                    observe(observe_graphics_tab_checkboxes),
                                ))
                                .insert_if(Checked, || {
                                    game_settings.fullscreen
                                });
                        });
                });
        }
        SelectedTabSettings::Controls => {}
    }
}

pub fn observe_graphics_tab_checkboxes(
    value_change: On<ValueChange<bool>>,
    graphics_checkboxes: Query<&GraphicsCheckbox>,
    mut commands: Commands,
    mut game_settings: ResMut<GameSettings>,
) {
    let source = value_change.source;

    let Ok(graphics_checkbox) = graphics_checkboxes.get(source) else {
        return;
    };

    let checked = value_change.value;

    match graphics_checkbox.0 {
        GraphicsCheckboxType::Fullscreen => {
            game_settings.fullscreen = checked;
            if checked {
                commands.entity(source).insert(Checked);
            } else {
                commands.entity(source).remove::<Checked>();
            }
        }
    }
}

fn handle_settings_menu_button_pressed(
    mut commands: Commands,
    query: Query<(&Interaction, &SettingsMenuButton), Changed<Interaction>>,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
    mut game_settings: ResMut<GameSettings>,
    mut apply_game_settings_message_writer: MessageWriter<
        ApplyGameSettingsMessage,
    >,
    graphics_checkboxes: Query<(Entity, &GraphicsCheckbox)>,
) {
    for (interaction, settings_menu_button) in query {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match &settings_menu_button.0 {
            SettingsButtonType::ToggleFullscreen => {
                game_settings.fullscreen = !game_settings.fullscreen;
                // TODO: this is pretty awkward
                let Some(graphics_checkbox) =
                    graphics_checkboxes.iter().find(|checkbox| {
                        checkbox.1.0 == GraphicsCheckboxType::Fullscreen
                    })
                else {
                    continue;
                };
                if game_settings.fullscreen {
                    commands.entity(graphics_checkbox.0).insert(Checked);
                } else {
                    commands.entity(graphics_checkbox.0).remove::<Checked>();
                }
            }
            SettingsButtonType::Back => {
                next_main_menu_state.set(MainMenuState::Root);
            }
            SettingsButtonType::Apply => {
                update_game_settings_file(&game_settings);
                apply_game_settings_message_writer
                    .write(ApplyGameSettingsMessage);
            }
        }
    }
}

fn handle_change_tab_button_pressed(
    query: Query<
        (&Interaction, &SettingsChangeTabButton),
        Changed<Interaction>,
    >,
    mut next_selected_tab: ResMut<NextState<SelectedTabSettings>>,
) {
    for (interaction, pressed_settings_change_tab_button) in query {
        let Interaction::Pressed = *interaction else {
            continue;
        };
        next_selected_tab.set(pressed_settings_change_tab_button.0.clone());
    }
}

fn update_settings_tab_button_color(
    query: Query<(&SettingsChangeTabButton, &mut BackgroundColor)>,
    settings_tab_state: Res<State<SelectedTabSettings>>,
) {
    for (button, mut background_color) in query {
        background_color.0 = if button.0 == *settings_tab_state.get() {
            UI_SELECTED
        } else {
            UI_PRIMARY
        };
    }
}

#[derive(Message)]
pub struct ApplyGameSettingsMessage;

fn apply_game_settings(
    mut message_reader: MessageReader<ApplyGameSettingsMessage>,
    game_settings: Res<GameSettings>,
    mut window: Single<&mut Window>,
    mut global_volume: ResMut<GlobalVolume>,
    mut music_audio_sinks: Query<&mut AudioSink, With<MusicAudio>>,
) {
    for _ in message_reader.read() {
        let fullscreen = game_settings.fullscreen;
        if fullscreen {
            window.mode =
                WindowMode::BorderlessFullscreen(MonitorSelection::Current);
        } else {
            window.mode = WindowMode::Windowed;
        }

        // TODO: should happen in audio plugin
        let master_volume = game_settings.master_volume;
        let new_master_volume =
            Volume::Linear((master_volume / 100.0).clamp(0.0, 1.0));
        global_volume.volume = new_master_volume;

        for mut music_audio_sink in &mut music_audio_sinks {
            music_audio_sink.set_volume(new_master_volume);
        }
    }
}
