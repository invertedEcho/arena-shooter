use bevy::prelude::*;

use crate::user_interface::{
    common::CommonUiPlugin, death_screen::DeathScreenPlugin,
    main_menu::MainMenuPlugin, pause_menu::PauseMenuPlugin,
    settings_menu::SettingsMenuPlugin,
};

mod common;
mod death_screen;
mod main_menu;
mod pause_menu;
mod settings_menu;

pub struct UserInterfacePlugin;

impl Plugin for UserInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PauseMenuPlugin)
            .add_plugins(CommonUiPlugin)
            .add_plugins(DeathScreenPlugin)
            .add_plugins(SettingsMenuPlugin)
            .add_plugins(MainMenuPlugin);
    }
}
