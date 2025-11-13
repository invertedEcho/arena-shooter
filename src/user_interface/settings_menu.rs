use crate::{
    game_flow::states::MainMenuState,
    user_interface::{
        DEFAULT_FONT_SIZE, DEFAULT_GAME_FONT_PATH,
        widgets::slider::{GameSettings, build_slider},
    },
};
use bevy::{
    prelude::*,
    ui_widgets::{ValueChange, observe},
    window::WindowMode,
};

pub struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MainMenuState::Settings), spawn_settings_menu)
            .add_systems(Update, handle_settings_menu_button_pressed);
    }
}

#[derive(Component)]
struct SettingsMenuButton(pub SettingsButtonType);

enum SettingsButtonType {
    ToggleFullscreen,
    Back,
}

#[derive(Component)]
struct SettingsMenuRoot;

fn spawn_settings_menu(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            SettingsMenuRoot,
            DespawnOnExit(MainMenuState::Settings),
        ))
        .with_children(|parent| {
            parent.spawn((
                build_slider(0.0, 100.0, 50.0),
                observe(
                    |value_change: On<ValueChange<f32>>,
                     mut widget_states: ResMut<GameSettings>| {
                        widget_states.volume = value_change.value;
                    },
                )
            ));
            parent.spawn((
                Node { ..default() },
                Button,
                SettingsMenuButton(SettingsButtonType::ToggleFullscreen),
                children![
                    Text::new("Toggle fullscreen"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: DEFAULT_FONT_SIZE,
                        ..default()
                    },
                ],
            ));
            parent
                .spawn((
                    Node {
                        padding: UiRect {
                            top: Val::Px(16.0),
                            ..default()
                        },
                        ..default()
                    },
                    Button,
                    SettingsMenuButton(SettingsButtonType::Back),
                ))
                .with_child((
                    Text::new("Back"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: DEFAULT_FONT_SIZE,
                        ..default()
                    },
                ));
        });
}

fn handle_settings_menu_button_pressed(
    mut window: Single<&mut Window>,
    query: Query<(&Interaction, &SettingsMenuButton), Changed<Interaction>>,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
) {
    for (interaction, settings_menu_button) in query {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match settings_menu_button.0 {
            SettingsButtonType::ToggleFullscreen => {
                let current_window_mode = window.mode;
                if current_window_mode == WindowMode::Windowed {
                    window.mode = WindowMode::BorderlessFullscreen(
                        MonitorSelection::Current,
                    );
                } else {
                    window.mode = WindowMode::Windowed;
                }
            }
            SettingsButtonType::Back => {
                next_main_menu_state.set(MainMenuState::Root);
            }
        }
    }
}
