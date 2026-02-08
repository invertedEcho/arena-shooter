use bevy::prelude::*;

use crate::user_interface::{
    common::CommonUiPlugin, death_screen::DeathScreenPlugin,
    disconnect_screen::DisconnectScreenPlugin,
    game_mode_selection::GameModeSelectionUIPlugin,
    loading_screen::LoadingScreenPlugin, main_menu::MainMenuPlugin,
    map_selection::MapSelectionPlugin, pause_menu::PauseMenuPlugin,
    score_board_overlay::ScoreBoardOverlayPlugin,
    settings_menu::SettingsMenuPlugin,
};

pub mod common;
mod death_screen;
mod disconnect_screen;
mod game_mode_selection;
mod loading_screen;
pub mod main_menu;
mod map_selection;
mod pause_menu;
mod score_board_overlay;
mod settings_menu;
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
            .add_plugins(MapSelectionPlugin)
            .add_plugins(DisconnectScreenPlugin)
            .add_plugins(LoadingScreenPlugin)
            .add_plugins(ScoreBoardOverlayPlugin);
    }
}
