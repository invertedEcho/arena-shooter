use bevy::prelude::*;

use crate::user_interface::{
    common::CommonUiPlugin, death_screen::DeathScreenPlugin,
    debug_overlay::DebugOverlayPlugin,
    game_mode_selection::GameModeSelectionUIPlugin, main_menu::MainMenuPlugin,
    map_selection::MapSelectionPlugin, pause_menu::PauseMenuPlugin,
    settings_menu::SettingsMenuPlugin,
};

mod common;
mod death_screen;
mod debug_overlay;
mod game_mode_selection;
pub mod main_menu;
mod map_selection;
mod pause_menu;
mod settings_menu;
mod widgets;

const AVA_FONT_PATH: &str = "fonts/AVA.ttf";
const DEFAULT_GAME_FONT_PATH: &str = "fonts/Ignotum/Ignotum-Regular.ttf";
pub const ITALIC_GAME_FONT_PATH: &str = "fonts/Ignotum/Ignotum-Italic.ttf";
const TITLE_FONT_SIZE: f32 = 64.0;
const DEFAULT_FONT_SIZE: f32 = 32.0;

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
            .add_plugins(MapSelectionPlugin);
    }
}
