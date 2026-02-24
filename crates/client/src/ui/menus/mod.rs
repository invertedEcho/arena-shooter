use bevy::prelude::*;

use crate::ui::menus::{
    credits::CreditsPlugin, game_mode_selection::GameModeSelectionUIPlugin,
    main_menu::MainMenuPlugin, map_selection::MapSelectionPlugin,
    settings_menu::SettingsMenuPlugin,
};

mod credits;
mod game_mode_selection;
mod main_menu;
mod map_selection;
mod settings_menu;

pub struct UIMenusPlugin;

impl Plugin for UIMenusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MainMenuPlugin)
            .add_plugins(CreditsPlugin)
            .add_plugins(GameModeSelectionUIPlugin)
            .add_plugins(SettingsMenuPlugin)
            .add_plugins(MapSelectionPlugin);
    }
}
