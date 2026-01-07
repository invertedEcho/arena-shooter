use bevy::prelude::*;

use crate::user_interface::{
    common::CommonUiPlugin, death_screen::DeathScreenPlugin,
    debug_overlay::DebugOverlayPlugin,
    disconnect_screen::DisconnectScreenPlugin,
    game_mode_selection::GameModeSelectionUIPlugin,
    loading_screen::LoadingScreenPlugin, main_menu::MainMenuPlugin,
    map_selection::MapSelectionPlugin, pause_menu::PauseMenuPlugin,
    settings_menu::SettingsMenuPlugin,
};

mod common;
mod death_screen;
mod debug_overlay;
mod disconnect_screen;
mod game_mode_selection;
mod loading_screen;
pub mod main_menu;
mod map_selection;
mod pause_menu;
mod settings_menu;
pub mod shared;
mod widgets;

pub struct UserInterfacePlugin;

impl Plugin for UserInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PauseMenuPlugin)
            .add_plugins(CommonUiPlugin)
            .add_plugins(DeathScreenPlugin)
            .add_plugins(SettingsMenuPlugin)
            .add_plugins(MainMenuPlugin)
            .add_plugins(GameModeSelectionUIPlugin)
            .add_plugins(DebugOverlayPlugin)
            .add_plugins(MapSelectionPlugin)
            .add_plugins(DisconnectScreenPlugin)
            .add_plugins(LoadingScreenPlugin);
    }
}
