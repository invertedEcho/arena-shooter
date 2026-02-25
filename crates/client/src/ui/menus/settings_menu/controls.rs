use bevy::prelude::*;

use crate::ui::{
    common::DEFAULT_GAME_FONT_PATH,
    menus::settings_menu::SettingsRightSideContentRoot,
};

pub fn spawn_controls_settings_tab_content(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings_right_side: Single<Entity, With<SettingsRightSideContentRoot>>,
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
                        Text::new("Nothing here yet..."),
                        TextFont {
                            font: font_handle.clone(),
                            ..default()
                        },
                    ));
                });
        });
}
