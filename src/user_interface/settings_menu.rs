use crate::{
    game_flow::states::MainMenuState,
    game_settings::GameSettings,
    user_interface::{
        DEFAULT_FONT_SIZE, DEFAULT_GAME_FONT_PATH,
        widgets::slider::{VolumeControlSlider, build_slider},
    },
};
use bevy::{
    color::palettes::{
        css::RED,
        tailwind::{PURPLE_800, SLATE_500, SLATE_600},
    },
    prelude::*,
    ui_widgets::{ValueChange, observe},
    window::WindowMode,
};

#[derive(SubStates, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[source(MainMenuState = MainMenuState::Settings)]
enum SettingsSelectedTab {
    #[default]
    Audio,
    Graphics,
    Controls,
}

pub struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SettingsSelectedTab>()
            .add_systems(OnEnter(MainMenuState::Settings), spawn_settings_menu)
            .add_systems(
                Update,
                handle_settings_tab_changed
                    .run_if(state_changed::<SettingsSelectedTab>),
            )
            .add_systems(Update, handle_settings_menu_button_pressed);
    }
}

#[derive(Component)]
struct SettingsMenuButton(pub SettingsButtonType);

enum SettingsButtonType {
    ChangeTab(SettingsSelectedTab),
    ToggleFullscreen,
    Back,
}

#[derive(Component)]
struct SettingsMenuRoot;

#[derive(Component)]
struct SettingsRightSideContentRoot;

fn spawn_settings_menu(asset_server: Res<AssetServer>, mut commands: Commands) {
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
            BorderColor::all(PURPLE_800),
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
                        align_items: AlignItems::End,
                        justify_content: JustifyContent::Start,
                        row_gap: px(16.0),
                        border: UiRect::all(px(2.0)),
                        ..default()
                    },
                    BorderColor::all(RED),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            padding: UiRect::all(px(8)),
                            width: percent(50),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(SLATE_500.into()),
                        Button,
                        SettingsMenuButton(SettingsButtonType::ChangeTab(
                            SettingsSelectedTab::Audio,
                        )),
                        children![Text::new("Audio")],
                    ));
                    parent.spawn((
                        Node {
                            padding: UiRect::all(px(8)),
                            width: percent(50),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        Button,
                        BackgroundColor(SLATE_500.into()),
                        SettingsMenuButton(SettingsButtonType::ChangeTab(
                            SettingsSelectedTab::Graphics,
                        )),
                        children![Text::new("Graphics")],
                    ));
                    parent.spawn((
                        Node {
                            padding: UiRect::all(px(8)),
                            width: percent(50),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        Button,
                        BackgroundColor(SLATE_500.into()),
                        SettingsMenuButton(SettingsButtonType::ChangeTab(
                            SettingsSelectedTab::Controls,
                        )),
                        children![Text::new("Controls")],
                    ));
                });
            parent
                .spawn((
                    Name::new("RightSideRoot"),
                    BorderColor::all(RED),
                    Node {
                        border: UiRect::all(px(2.0)),
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
                        ..default()
                    },
                    Name::new("RightSideContentRoot"),
                    SettingsRightSideContentRoot,
                ));
        })
        .with_child((
            Node {
                padding: UiRect {
                    top: Val::Px(16.0),
                    ..default()
                },
                ..default()
            },
            Button,
            BackgroundColor(SLATE_500.into()),
            SettingsMenuButton(SettingsButtonType::Back),
            children![
                Text::new("Back"),
                TextFont {
                    font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                    font_size: DEFAULT_FONT_SIZE,
                    ..default()
                },
            ],
        ));
}

// We sadly cant use a function which gets the required components depending on the selected tab,
// as it causes issues with arms having different types even if they are all of type Bundle
fn handle_settings_tab_changed(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    settings_right_side: Single<Entity, With<SettingsRightSideContentRoot>>,
    settings_tab_state: Res<State<SettingsSelectedTab>>,
) {
    commands.entity(*settings_right_side).despawn_children();
    let settings_tab_state = settings_tab_state.get();
    match settings_tab_state {
        SettingsSelectedTab::Audio => {
            commands
                .entity(*settings_right_side)
                .with_children(|parent| {
                    parent.spawn((
                        Node { ..default() },
                        children![Text::new("Master Volume")],
                    ));
                    parent.spawn((
                        build_slider(0.0, 100.0, 50.0, VolumeControlSlider),
                        observe(observe_volume_control_slider_change),
                    ));
                });
        }
        SettingsSelectedTab::Graphics => {
            commands
                .entity(*settings_right_side)
                .with_children(|parent| {
                    parent.spawn((
                        Node { ..default() },
                        Button,
                        BackgroundColor(SLATE_500.into()),
                        SettingsMenuButton(
                            SettingsButtonType::ToggleFullscreen,
                        ),
                        children![
                            Text::new("Toggle fullscreen"),
                            TextFont {
                                font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                                font_size: DEFAULT_FONT_SIZE,
                                ..default()
                            },
                        ],
                    ));
                });
        }
        SettingsSelectedTab::Controls => {}
    }
}

fn handle_settings_menu_button_pressed(
    mut window: Single<&mut Window>,
    query: Query<(&Interaction, &SettingsMenuButton), Changed<Interaction>>,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
    mut game_settings: ResMut<GameSettings>,
    mut next_selected_tab: ResMut<NextState<SettingsSelectedTab>>,
) {
    for (interaction, settings_menu_button) in query {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match &settings_menu_button.0 {
            SettingsButtonType::ToggleFullscreen => {
                let mut fullscreen = false;
                // TODO: move this outside and use a system that watches gamesettings
                let current_window_mode = window.mode;
                if current_window_mode == WindowMode::Windowed {
                    window.mode = WindowMode::BorderlessFullscreen(
                        MonitorSelection::Current,
                    );
                    fullscreen = true;
                } else {
                    window.mode = WindowMode::Windowed;
                }
                game_settings.fullscreen = fullscreen;
            }
            SettingsButtonType::Back => {
                next_main_menu_state.set(MainMenuState::Root);
            }
            SettingsButtonType::ChangeTab(selected_tab) => {
                next_selected_tab.set(selected_tab.clone());
            }
        }
    }
}

fn observe_volume_control_slider_change(
    value_change: On<ValueChange<f32>>,
    mut widget_states: ResMut<GameSettings>,
) {
    widget_states.audio_volume = value_change.value;
}
