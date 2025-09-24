use bevy::prelude::*;

use crate::user_interface::{
    common::CommonUiPlugin, death_screen::DeathScreenPlugin,
    debug_overlay::DebugOverlayPlugin,
    game_mode_selection::GameModeSelectionUIPlugin, main_menu::MainMenuPlugin,
    pause_menu::PauseMenuPlugin, settings_menu::SettingsMenuPlugin,
};

mod common;
mod death_screen;
mod debug_overlay;
mod game_mode_selection;
pub mod main_menu;
mod pause_menu;
mod settings_menu;

pub struct UserInterfacePlugin;

impl Plugin for UserInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PauseMenuPlugin)
            .add_plugins(CommonUiPlugin)
            .add_plugins(DeathScreenPlugin)
            .add_plugins(SettingsMenuPlugin)
            .add_plugins(MainMenuPlugin)
            .add_plugins(GameModeSelectionUIPlugin)
            .add_plugins(DebugOverlayPlugin);
    }
}
