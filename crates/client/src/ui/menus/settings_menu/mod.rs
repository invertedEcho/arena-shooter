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
                spawn_audio_settings_tab_content, update_volume_slider_value,
            },
            controls::spawn_controls_settings_tab_content,
            graphics::{
                handle_graphics_game_settings_change,
                handle_graphics_setting_button_press,
                spawn_graphics_settings_tab_content,
            },
        },
        widgets::button::build_common_button,
    },
};
use bevy::prelude::*;

mod audio;
mod controls;
mod graphics;

pub struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SettingsCurrentTab>();
        app.add_systems(
            OnEnter(MainMenuState::Settings),
            (spawn_settings_menu, spawn_audio_settings_tab_content).chain(),
        );
        app.add_systems(
            OnEnter(SettingsCurrentTab::Audio),
            spawn_audio_settings_tab_content,
        );
        app.add_systems(
            OnEnter(SettingsCurrentTab::Graphics),
            spawn_graphics_settings_tab_content,
        );
        app.add_systems(
            OnEnter(SettingsCurrentTab::Controls),
            spawn_controls_settings_tab_content,
        );
        app.add_systems(
            Update,
            update_settings_tab_button_color
                .run_if(state_changed::<SettingsCurrentTab>),
        );
        app.add_systems(
            Update,
            (
                handle_settings_menu_action_button_pressed,
                update_volume_slider_value,
                handle_change_tab_button_pressed,
                handle_graphics_setting_button_press,
            ),
        );
        app.add_systems(
            Update,
            handle_graphics_game_settings_change
                .run_if(resource_changed::<GameSettings>),
        );
    }
}

#[derive(SubStates, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[source(MainMenuState = MainMenuState::Settings)]
pub enum SettingsCurrentTab {
    #[default]
    Audio,
    Graphics,
    Controls,
}

#[derive(Component)]
enum SettingsMenuActionButton {
    Back,
    Save,
}

#[derive(Component)]
pub struct SettingsTabButton(pub SettingsCurrentTab);

#[derive(Component)]
struct SettingsMenuRoot;

#[derive(Component)]
struct SettingsRightSideContentRoot;

fn spawn_settings_menu(asset_server: Res<AssetServer>, mut commands: Commands) {
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
            build_settings_content_container(),
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
                SettingsMenuActionButton::Save
            ),
            build_common_button(
                "Back",
                font_handle,
                SettingsMenuActionButton::Back
            )
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
                SettingsTabButton(SettingsCurrentTab::Audio),
                true
            ),
            build_tab_button_settings(
                "Graphics",
                font_handle.clone(),
                SettingsTabButton(SettingsCurrentTab::Graphics),
                false
            ),
            build_tab_button_settings(
                "Controls",
                font_handle.clone(),
                SettingsTabButton(SettingsCurrentTab::Controls),
                false
            ),
        ],
    )
}

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

fn build_settings_content_container() -> impl Bundle {
    (
        Name::new("Settings Content Container"),
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(16.0),
            padding: UiRect::all(px(16.0)),
            ..default()
        },
        BackgroundColor(UI_PANEL),
        SettingsRightSideContentRoot,
    )
}

fn handle_settings_menu_action_button_pressed(
    query: Query<
        (&Interaction, &SettingsMenuActionButton),
        Changed<Interaction>,
    >,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
    game_settings: Res<GameSettings>,
) {
    for (interaction, settings_menu_button) in query {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match &settings_menu_button {
            SettingsMenuActionButton::Back => {
                next_main_menu_state.set(MainMenuState::Root);
            }
            SettingsMenuActionButton::Save => {
                update_game_settings_file(&game_settings);
            }
        }
    }
}

fn handle_change_tab_button_pressed(
    query: Query<(&Interaction, &SettingsTabButton), Changed<Interaction>>,
    mut next_selected_tab: ResMut<NextState<SettingsCurrentTab>>,
) {
    for (interaction, pressed_settings_change_tab_button) in query {
        let Interaction::Pressed = *interaction else {
            continue;
        };
        next_selected_tab.set(pressed_settings_change_tab_button.0.clone());
    }
}

fn update_settings_tab_button_color(
    query: Query<(&SettingsTabButton, &mut BackgroundColor)>,
    settings_tab_state: Res<State<SettingsCurrentTab>>,
) {
    for (button, mut background_color) in query {
        background_color.0 = if button.0 == *settings_tab_state.get() {
            UI_SELECTED
        } else {
            UI_PANEL
        };
    }
}
