use bevy::{
    dev_tools::fps_overlay::FpsOverlayConfig,
    prelude::*,
    ui::Checked,
    ui_widgets::{ValueChange, observe},
    window::WindowMode,
};

use crate::{
    game_settings::GameSettings,
    ui::{
        common::{DEFAULT_FONT_SIZE, DEFAULT_GAME_FONT_PATH},
        menus::settings_menu::SettingsRightSideContentRoot,
        widgets::checkbox::build_checkbox,
    },
};

#[derive(Component)]
pub struct GraphicsCheckbox(GraphicsSettingType);

#[derive(Component)]
pub struct GraphicsButton(GraphicsSettingType);

#[derive(Component, PartialEq)]
enum GraphicsSettingType {
    BorderlessFullscreen,
    FpsOverlayShown,
}

pub fn spawn_graphics_settings_tab_content(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings_right_side: Single<Entity, With<SettingsRightSideContentRoot>>,
    game_settings: Res<GameSettings>,
) {
    commands.entity(*settings_right_side).despawn_children();

    let font_handle = asset_server.load(DEFAULT_GAME_FONT_PATH);

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
                        Button,
                        GraphicsButton(
                            GraphicsSettingType::BorderlessFullscreen,
                        ),
                        children![
                            Text::new("Borderless Fullscreen"),
                            TextFont {
                                font: font_handle.clone(),
                                font_size: DEFAULT_FONT_SIZE,
                                ..default()
                            }
                        ],
                    ));
                    parent
                        .spawn((
                            build_checkbox(
                                font_handle.clone(),
                                "",
                                GraphicsCheckbox(
                                    GraphicsSettingType::BorderlessFullscreen,
                                ),
                            ),
                            observe(observe_graphics_tab_checkboxes),
                        ))
                        .insert_if(Checked, || {
                            game_settings.graphics.borderless_fullscreen
                        });
                });
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: percent(100.0),
                            ..default()
                        },
                        Button,
                        GraphicsButton(GraphicsSettingType::FpsOverlayShown),
                        children![
                            Text::new("FPS Overlay shown"),
                            TextFont {
                                font: font_handle.clone(),
                                font_size: DEFAULT_FONT_SIZE,
                                ..default()
                            }
                        ],
                    ));
                    parent
                        .spawn((
                            build_checkbox(
                                font_handle.clone(),
                                "",
                                GraphicsCheckbox(
                                    GraphicsSettingType::FpsOverlayShown,
                                ),
                            ),
                            observe(observe_graphics_tab_checkboxes),
                        ))
                        .insert_if(Checked, || {
                            game_settings.graphics.fps_overlay_shown
                        });
                });
        });
}

fn observe_graphics_tab_checkboxes(
    value_change: On<ValueChange<bool>>,
    graphics_checkboxes: Query<&GraphicsCheckbox>,
    mut commands: Commands,
    mut game_settings: ResMut<GameSettings>,
) {
    let source = value_change.source;
    info!("Some checkbox got its value changed!");

    let Ok(graphics_checkbox) = graphics_checkboxes.get(source) else {
        warn!("But we couldnt find the checkbox it came from?");
        return;
    };

    let checked = value_change.value;

    match graphics_checkbox.0 {
        GraphicsSettingType::BorderlessFullscreen => {
            game_settings.graphics.borderless_fullscreen = checked;
            if checked {
                commands.entity(source).insert(Checked);
            } else {
                commands.entity(source).remove::<Checked>();
            }
        }
        GraphicsSettingType::FpsOverlayShown => {
            game_settings.graphics.fps_overlay_shown = checked;
            if checked {
                commands.entity(source).insert(Checked);
            } else {
                commands.entity(source).remove::<Checked>();
            }
        }
    }
}

pub fn handle_graphics_game_settings_change(
    mut commands: Commands,
    game_settings: Res<GameSettings>,
    mut fps_overlay: ResMut<FpsOverlayConfig>,
    mut window: Single<&mut Window>,
    graphics_checkboxes: Query<(Entity, &GraphicsCheckbox)>,
) {
    fps_overlay.enabled = game_settings.graphics.fps_overlay_shown;
    if let Some((entity, _)) =
        graphics_checkboxes.iter().find(|(_, checkbox)| {
            checkbox.0 == GraphicsSettingType::FpsOverlayShown
        })
    {
        if fps_overlay.enabled {
            commands.entity(entity).insert(Checked);
        } else {
            commands.entity(entity).remove::<Checked>();
        }
    }

    let fullscreen = game_settings.graphics.borderless_fullscreen;
    if fullscreen {
        window.mode =
            WindowMode::BorderlessFullscreen(MonitorSelection::Current);
    } else {
        window.mode = WindowMode::Windowed;
    }

    if let Some((entity, _)) =
        graphics_checkboxes.iter().find(|(_, checkbox)| {
            checkbox.0 == GraphicsSettingType::BorderlessFullscreen
        })
    {
        if fullscreen {
            commands.entity(entity).insert(Checked);
        } else {
            commands.entity(entity).remove::<Checked>();
        }
    }
}

pub fn handle_graphics_setting_button_press(
    query: Query<(&Interaction, &GraphicsButton), Changed<Interaction>>,
    mut game_settings: ResMut<GameSettings>,
) {
    for (interaction, graphics_button) in query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match graphics_button.0 {
            GraphicsSettingType::BorderlessFullscreen => {
                game_settings.graphics.borderless_fullscreen =
                    !game_settings.graphics.borderless_fullscreen;
            }
            GraphicsSettingType::FpsOverlayShown => {
                game_settings.graphics.fps_overlay_shown =
                    !game_settings.graphics.fps_overlay_shown;
            }
        }
    }
}
