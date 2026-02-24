use bevy::prelude::*;

use crate::ui::{
    common::CommonUiPlugin, hud::PlayerHudPlugin, menus::UIMenusPlugin,
    screens::UIScreensPlugin,
};

pub mod common;
mod hud;
mod menus;
mod screens;
pub mod widgets;

// idk i kinda dont like this but no idea how else i should do it so systems from different modules
// can react to this resource change
// in future more stuff will be added here
#[derive(Resource, Default)]
pub struct UiState {
    pub score_board_overlay_visible: bool,
}

pub struct UserInterfacePlugin;

impl Plugin for UserInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerHudPlugin)
            .add_plugins(UIMenusPlugin)
            .add_plugins(CommonUiPlugin)
            .add_plugins(UIScreensPlugin);

        app.insert_resource(UiState::default());
    }
}
