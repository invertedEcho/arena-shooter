use crate::{
    game_flow::states::MainMenuState,
    game_settings::{GameSettings, update_game_settings_file},
    ui::{
        common::{
            DEFAULT_GAME_FONT_PATH, UI_BACKGROUND, UI_BORDER, UI_PANEL,
            UI_SELECTED, UI_TEXT,
        },
        menus::settings_menu::{
            audio::{
                build_audio_settings_tab_content, update_volume_slider_value,
            },
            graphics::{GraphicsCheckbox, GraphicsCheckboxType},
        },
        widgets::{button::build_common_button, checkbox::build_checkbox},
    },
};
use bevy::{
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
        app.init_state::<CurrentTabSettings>()
            .add_message::<ApplyGameSettingsMessage>()
            .add_systems(OnEnter(MainMenuState::Settings), spawn_settings_menu)
            .add_systems(
                Update,
                (
                    handle_settings_tab_changed,
                    update_settings_tab_button_color,
                )
                    .run_if(state_changed::<CurrentTabSettings>),
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
pub enum CurrentTabSettings {
    #[default]
    Audio,
    Graphics,
    Controls,
}

#[derive(Component)]
enum SettingsMenuButton {
    ToggleFullscreen,
    Back,
    Save,
}

#[derive(Component)]
pub struct TabButtonSettings(pub CurrentTabSettings);

#[derive(Component)]
struct SettingsMenuRoot;

#[derive(Component)]
struct SettingsRightSideContentRoot;

fn spawn_settings_menu(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    game_settings: Res<GameSettings>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(16.0),
            padding: UiRect::all(percent(2.0)),
            ..default()
        },
        BackgroundColor(UI_BACKGROUND),
        SettingsMenuRoot,
        Name::new("Settings Menu Root Node"),
        DespawnOnExit(MainMenuState::Settings),
        children![
            build_tab_buttons(asset_server.load(DEFAULT_GAME_FONT_PATH)),
            build_settings_content_container(
                asset_server.load(DEFAULT_GAME_FONT_PATH),
                game_settings
            ),
            build_settings_action_container(
                asset_server.load(DEFAULT_GAME_FONT_PATH)
            ),
        ],
    ));
}

fn build_settings_action_container(font_handle: Handle<Font>) -> impl Bundle {
    (
        Node {
            justify_content: JustifyContent::End,
            flex_direction: FlexDirection::Row,
            column_gap: px(32.0),
            ..default()
        },
        children![
            build_common_button(
                "Save",
                font_handle.clone(),
                SettingsMenuButton::Save
            ),
            build_common_button("Back", font_handle, SettingsMenuButton::Back)
        ],
    )
}

fn build_tab_buttons(font_handle: Handle<Font>) -> impl Bundle {
    (
        Node {
            justify_content: JustifyContent::SpaceBetween,
            width: percent(100.0),
            ..default()
        },
        children![
            build_tab_button_settings(
                "Audio",
                font_handle.clone(),
                TabButtonSettings(CurrentTabSettings::Audio),
                true
            ),
            build_tab_button_settings(
                "Graphics",
                font_handle.clone(),
                TabButtonSettings(CurrentTabSettings::Graphics),
                false
            ),
            build_tab_button_settings(
                "Controls",
                font_handle.clone(),
                TabButtonSettings(CurrentTabSettings::Controls),
                false
            ),
        ],
    )
}

// fn build_left_side(font_handle: Handle<Font>) -> impl Bundle {
//     (
//         Name::new("Left Side Root"),
//         Node {
//             width: percent(25.0),
//             height: percent(100.0),
//             flex_direction: FlexDirection::Column,
//             align_items: AlignItems::End,
//             row_gap: px(8.0),
//             padding: UiRect::all(px(8.0)),
//             justify_content: JustifyContent::SpaceBetween,
//             ..default()
//         },
//         BackgroundColor(UI_PANEL),
//         children![
//             (
//                 Node {
//                     justify_content: JustifyContent::End,
//                     flex_direction: FlexDirection::Column,
//                     width: percent(100.0),
//                     row_gap: DEFAULT_ROW_GAP,
//                     ..default()
//                 },
//                 children![
//                     build_tab_button_settings(
//                         "Audio",
//                         font_handle.clone(),
//                         TabButtonSettings(CurrentTabSettings::Audio)
//                     ),
//                     build_tab_button_settings(
//                         "Graphics",
//                         font_handle.clone(),
//                         TabButtonSettings(CurrentTabSettings::Graphics)
//                     ),
//                     build_tab_button_settings(
//                         "Controls",
//                         font_handle.clone(),
//                         TabButtonSettings(CurrentTabSettings::Controls)
//                     ),
//                 ]
//             ),
//             (
//                 Node {
//                     justify_content: JustifyContent::End,
//                     flex_direction: FlexDirection::Column,
//                     width: percent(100.0),
//                     row_gap: DEFAULT_ROW_GAP,
//                     ..default()
//                 },
//                 children![
//                     build_tab_button_settings(
//                         "Save",
//                         font_handle.clone(),
//                         SettingsMenuButton::Save
//                     ),
//                     build_tab_button_settings(
//                         "Back",
//                         font_handle,
//                         SettingsMenuButton::Back
//                     ),
//                 ]
//             )
//         ],
//     )
// }

fn build_tab_button_settings<T: Component>(
    button_text: &str,
    font_handle: Handle<Font>,
    marker_component: T,
    selected: bool,
) -> impl Bundle {
    (
        Node {
            padding: UiRect::all(px(8)),
            justify_content: JustifyContent::Center,
            border: UiRect::left(px(4.0)),
            width: percent(100.0),
            ..default()
        },
        Button,
        BackgroundColor(get_background_color_tab_button(selected)),
        BorderColor::all(UI_BORDER),
        marker_component,
        children![(
            Text::new(button_text),
            TextFont {
                font: font_handle,
                ..default()
            },
            TextColor(UI_TEXT)
        )],
    )
}

fn get_background_color_tab_button(selected: bool) -> Color {
    if selected { UI_SELECTED } else { UI_PANEL }
}

fn build_settings_content_container(
    font_handle: Handle<Font>,
    game_settings: Res<GameSettings>,
) -> impl Bundle {
    (
        Name::new("Settings Content Container"),
        Node {
            width: percent(100.0),
            height: percent(100.0),
            ..default()
        },
        BackgroundColor(UI_PANEL),
        children![(
            build_audio_settings_tab_content(font_handle, game_settings),
            SettingsRightSideContentRoot
        )],
    )
}

// We sadly cant use a function which gets the required components depending on the selected tab,
// as it causes issues with arms having different types even if they are all of type Bundle
fn handle_settings_tab_changed(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    settings_right_side: Single<Entity, With<SettingsRightSideContentRoot>>,
    settings_tab_state: Res<State<CurrentTabSettings>>,
    game_settings: Res<GameSettings>,
) {
    commands.entity(*settings_right_side).despawn_children();
    let settings_tab_state = settings_tab_state.get();
    match settings_tab_state {
        CurrentTabSettings::Audio => {
            commands.entity(*settings_right_side).with_child(
                build_audio_settings_tab_content(
                    asset_server.load(DEFAULT_GAME_FONT_PATH),
                    game_settings,
                ),
            );
        }
        CurrentTabSettings::Graphics => {
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
                                SettingsMenuButton::ToggleFullscreen,
                                children![
                                    Text::new("Borderless Fullscreen"),
                                    TextFont {
                                        font: asset_server
                                            .load(DEFAULT_GAME_FONT_PATH),
                                        ..default()
                                    }
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
        CurrentTabSettings::Controls => {}
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
        match &settings_menu_button {
            SettingsMenuButton::ToggleFullscreen => {
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
            SettingsMenuButton::Back => {
                next_main_menu_state.set(MainMenuState::Root);
            }
            SettingsMenuButton::Save => {
                update_game_settings_file(&game_settings);
                apply_game_settings_message_writer
                    .write(ApplyGameSettingsMessage);
            }
        }
    }
}

fn handle_change_tab_button_pressed(
    query: Query<(&Interaction, &TabButtonSettings), Changed<Interaction>>,
    mut next_selected_tab: ResMut<NextState<CurrentTabSettings>>,
) {
    for (interaction, pressed_settings_change_tab_button) in query {
        let Interaction::Pressed = *interaction else {
            continue;
        };
        next_selected_tab.set(pressed_settings_change_tab_button.0.clone());
    }
}

fn update_settings_tab_button_color(
    query: Query<(&TabButtonSettings, &mut BackgroundColor)>,
    settings_tab_state: Res<State<CurrentTabSettings>>,
) {
    for (button, mut background_color) in query {
        background_color.0 = if button.0 == *settings_tab_state.get() {
            UI_SELECTED
        } else {
            UI_PANEL
        };
    }
}

#[derive(Message)]
pub struct ApplyGameSettingsMessage;

// TODO: should happen in a graphics/window file or something
fn apply_game_settings(
    mut message_reader: MessageReader<ApplyGameSettingsMessage>,
    game_settings: Res<GameSettings>,
    mut window: Single<&mut Window>,
) {
    for _ in message_reader.read() {
        let fullscreen = game_settings.fullscreen;
        if fullscreen {
            window.mode =
                WindowMode::BorderlessFullscreen(MonitorSelection::Current);
        } else {
            window.mode = WindowMode::Windowed;
        }
    }
}
